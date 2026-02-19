use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};
use crate::navigation::{navigate, AppRoute};
use crate::stores::get_auth_store;

/// A pending attachment that shows a preview immediately while uploading in the background.
#[derive(Clone, PartialEq)]
struct PendingAttachment {
    local_id: String,
    filename: String,
    mime: String,
    size: u64,
    /// Local temp file path for immediate image preview (before upload completes).
    local_preview: String,
    server: Option<rorumall_shared::Attachment>,
}

#[component]
pub fn message_input(channel_id: String, group_id: String, host: String) -> NodeHandle {
    let input_text = use_signal(|| String::new());
    let message_type = use_signal(|| "message".to_string());
    let reply_to = use_signal(|| None::<String>);
    let pending = use_signal(|| Vec::<PendingAttachment>::new());

    let cid = use_signal(|| channel_id.clone());
    let h = use_signal(|| host.clone());

    // Keyboard interceptor for clipboard image paste (Ctrl+V)
    use_mount(move || {
        tracing::info!("Setting keyboard interceptor for clipboard paste");
        rinch_core::set_keyboard_interceptor(move |key_data| {
            tracing::info!("Keyboard interceptor called: key={}, ctrl={}", key_data.key, key_data.ctrl);
            if key_data.ctrl && key_data.key == "v" {
                // Try to get PNG bytes from clipboard (arboard first, then wl-paste fallback)
                let png_bytes = get_clipboard_image_png();
                if let Some(png_bytes) = png_bytes {
                    // Encode as data URI for immediate synchronous preview in rinch
                    let local_id = uuid::Uuid::new_v4().to_string();
                    let preview_path = encode_data_uri(&png_bytes, "image/png");
                    let size = png_bytes.len() as u64;
                    pending.update(|atts| atts.push(PendingAttachment {
                        local_id: local_id.clone(),
                        filename: "clipboard-image.png".to_string(),
                        mime: "image/png".to_string(),
                        size,
                        local_preview: preview_path,
                        server: None,
                    }));

                    // Upload in background
                    let client = get_auth_store().make_client();
                    crate::runtime::spawn(
                        async move {
                            client
                                .upload_file(png_bytes, "clipboard-image.png", "image/png")
                                .await
                        },
                        move |result| match result {
                            Ok(attachment) => {
                                pending.update(|atts| {
                                    if let Some(pa) = atts.iter_mut().find(|a| a.local_id == local_id) {
                                        pa.server = Some(attachment);
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Clipboard image upload failed: {}", e);
                                pending.update(|atts| atts.retain(|a| a.local_id != local_id));
                            }
                        },
                    );
                    return true;
                }
            }
            false
        });

        move || {
            rinch_core::clear_keyboard_interceptor();
        }
    });

    let on_send = move || {
        let text = input_text.get().clone();
        if text.trim().is_empty() && pending.get().is_empty() {
            return;
        }

        // Only send attachments that have finished uploading
        let attachments: Vec<rorumall_shared::Attachment> = pending.get()
            .iter()
            .filter_map(|pa| pa.server.clone())
            .collect();

        // Don't send if any are still uploading
        if attachments.len() != pending.get().len() {
            tracing::warn!("Some attachments still uploading, please wait");
            return;
        }

        let domain = get_auth_store().domain();
        let h_val = h.get().clone();
        let ws_host = if h_val.is_empty() { domain } else { h_val };
        let cid_val = cid.get().clone();

        if let Some(handle) = crate::ws::get_handle(&ws_host) {
            let nonce = uuid::Uuid::new_v4().to_string();
            let mt = match message_type.get().as_str() {
                "memo" => Some(rorumall_shared::MessageType::Memo),
                "article" => Some(rorumall_shared::MessageType::Article),
                _ => None,
            };

            let parent = reply_to.get().clone();

            let result = if let Some(pid) = parent {
                handle.send_reply(&cid_val, &text, &nonce, &pid, mt, attachments)
            } else {
                handle.send_message_with_options(&cid_val, &text, &nonce, None, mt, attachments)
            };

            match result {
                Ok(()) => {
                    input_text.set(String::new());
                    reply_to.set(None);
                    pending.set(Vec::new());
                }
                Err(e) => {
                    tracing::error!("Failed to send message: {}", e);
                }
            }
        }
    };

    let host_for_article = host.clone();
    let gid_for_article = group_id.clone();
    let cid_for_article = channel_id.clone();

    let on_file_pick = move || {
        let path = rinch::dialogs::open_file()
            .add_filter("All files", &["*"])
            .pick_file();
        if let Some(path) = path {
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".to_string());
            let mime = mime_from_extension(&filename);

            // Read file size for the preview
            let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

            // Add preview immediately
            let local_id = uuid::Uuid::new_v4().to_string();
            // For image files, use the original file path as local preview
            let local_preview = if mime.starts_with("image/") {
                path.to_string_lossy().to_string()
            } else {
                String::new()
            };
            pending.update(|atts| atts.push(PendingAttachment {
                local_id: local_id.clone(),
                filename: filename.clone(),
                mime: mime.clone(),
                size: file_size,
                local_preview,
                server: None,
            }));

            // Upload in background
            let client = get_auth_store().make_client();
            crate::runtime::spawn(
                async move {
                    let data = std::fs::read(&path)
                        .map_err(|e| rorumall_shared::ApiError::Network(e.to_string()))?;
                    client.upload_file(data, &filename, &mime).await
                },
                move |result| match result {
                    Ok(attachment) => {
                        pending.update(|atts| {
                            if let Some(pa) = atts.iter_mut().find(|a| a.local_id == local_id) {
                                pa.server = Some(attachment);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Upload failed: {}", e);
                        pending.update(|atts| atts.retain(|a| a.local_id != local_id));
                    }
                },
            );
        }
    };

    rsx! {
        div {
            class: "message-input-area",
            style: "border-top: 1px solid var(--rinch-color-dark-4, #373a40); padding: 12px 16px; background: var(--rinch-color-dark-7, #1a1b1e);",

            // Reply indicator
            if reply_to.get().is_some() {
                Group {
                    gap: "xs",
                    style: "margin-bottom: 8px;",

                    Text {
                        size: "xs",
                        color: "dimmed",
                        {|| { let p = reply_to.get().clone().unwrap_or_default(); format!("Replying to {}...", &p[..8.min(p.len())]) }}
                    }

                    ActionIcon {
                        variant: "subtle",
                        size: "xs",
                        onclick: move || reply_to.set(None),
                        {render_tabler_icon(__scope, TablerIcon::X, TablerIconStyle::Outline)}
                    }
                }
            }

            // Pending attachments with instant preview
            if !pending.get().is_empty() {
                div {
                    style: "display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 8px;",

                    for pa in pending.get().clone() {
                        div {
                            key: pa.local_id.clone(),
                            {pending_attachment_preview(__scope, pa, pending)}
                        }
                    }
                }
            }

            Group {
                gap: "sm",

                // Message type selector
                ActionIcon {
                    variant: "subtle",
                    onclick: move || {
                        let current = message_type.get().clone();
                        let next = match current.as_str() {
                            "message" => "memo",
                            "memo" => "message",
                            _ => "message",
                        };
                        message_type.set(next.to_string());
                    },
                    {match message_type.get().as_str() {
                        "memo" => render_tabler_icon(__scope, TablerIcon::Note, TablerIconStyle::Outline),
                        _ => render_tabler_icon(__scope, TablerIcon::Message, TablerIconStyle::Outline),
                    }}
                }

                // File attachment
                ActionIcon {
                    variant: "subtle",
                    onclick: move || on_file_pick(),
                    {render_tabler_icon(__scope, TablerIcon::Paperclip, TablerIconStyle::Outline)}
                }

                // Article compose
                ActionIcon {
                    variant: "subtle",
                    onclick: move || navigate(AppRoute::ComposeArticle {
                        host: host_for_article.clone(),
                        group_id: gid_for_article.clone(),
                        channel_id: cid_for_article.clone(),
                    }),
                    {render_tabler_icon(__scope, TablerIcon::Article, TablerIconStyle::Outline)}
                }

                // Text input
                TextInput {
                    placeholder: "Type a message...",
                    style: "flex: 1;",
                    value_fn: move || input_text.get().clone(),
                    oninput: move |val: String| input_text.set(val),
                    onsubmit: move || on_send(),
                }

                // Send button
                ActionIcon {
                    variant: "filled",
                    color: "indigo",
                    onclick: move || on_send(),
                    {render_tabler_icon(__scope, TablerIcon::Send, TablerIconStyle::Outline)}
                }
            }
        }
    }
}

#[component]
fn pending_attachment_preview(
    pa: PendingAttachment,
    pending: Signal<Vec<PendingAttachment>>,
) -> NodeHandle {
    let is_image = pa.mime.starts_with("image/");
    let uploading = pa.server.is_none();
    let size = pa.size;
    let local_id = use_signal(|| pa.local_id.clone());
    let filename = use_signal(|| pa.filename.clone());

    // Use local file for immediate preview, or server URL once uploaded
    let preview_src = use_signal(|| if !pa.local_preview.is_empty() {
        pa.local_preview.clone()
    } else {
        pa.server.as_ref().map(|s| s.url.clone()).unwrap_or_default()
    });

    rsx! {
        div {
            style: "position: relative; display: inline-block;",

            if is_image && !preview_src.get().is_empty() {
                img {
                    src: {preview_src.get().clone()},
                    width: "80",
                    height: "80",
                    style: "border-radius: 4px; display: block; object-fit: cover;",
                    alt: "attachment",
                }
            } else if is_image {
                div {
                    style: "width: 100px; height: 70px; display: flex; align-items: center; justify-content: center; background: #2c2e33; border-radius: 4px;",
                    {render_tabler_icon(__scope, TablerIcon::Photo, TablerIconStyle::Outline)}
                }
            } else {
                div {
                    style: "display: flex; align-items: center; gap: 4px; padding: 8px; background: #242424; border-radius: 4px;",
                    {render_tabler_icon(__scope, TablerIcon::File, TablerIconStyle::Outline)}
                    Text {
                        size: "xs",
                        color: "dimmed",
                        {format!("{} ({})", truncate_filename(&filename.get(), 12), format_size(size))}
                    }
                }
            }

            if uploading {
                div {
                    style: "position: absolute; bottom: 2px; left: 2px;",
                    Loader { size: "xs" }
                }
            }

            Text {
                size: "xs",
                color: "dimmed",
                {format_size(size)}
            }

            // Remove button
            div {
                style: "position: absolute; top: -4px; right: -4px;",

                ActionIcon {
                    variant: "filled",
                    color: "red",
                    size: "xs",
                    radius: "xl",
                    onclick: move || {
                        let rid = local_id.get().clone();
                        pending.update(|atts| { atts.retain(|a| a.local_id != rid); });
                    },
                    {render_tabler_icon(__scope, TablerIcon::X, TablerIconStyle::Outline)}
                }
            }
        }
    }
}

/// Encode raw bytes as a base64 data URI for inline image display.
fn encode_data_uri(bytes: &[u8], mime: &str) -> String {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!("data:{};base64,{}", mime, b64)
}

/// Try to get an image from the clipboard as PNG bytes.
/// Attempts arboard (X11) first, then falls back to `wl-paste` for Wayland.
fn get_clipboard_image_png() -> Option<Vec<u8>> {
    tracing::info!("get_clipboard_image_png: trying arboard...");
    // Try arboard first (works on X11 and macOS/Windows)
    match rinch::clipboard::paste_image() {
        Ok(img_data) => {
            tracing::info!("arboard: got image {}x{}", img_data.width, img_data.height);
            if let Some(png) = encode_rgba_to_png(
                img_data.width as u32,
                img_data.height as u32,
                &img_data.bytes,
            ) {
                tracing::info!("arboard: encoded to {} bytes PNG", png.len());
                return Some(png);
            }
        }
        Err(e) => {
            tracing::info!("arboard failed: {}", e);
        }
    }

    tracing::info!("get_clipboard_image_png: trying wl-paste...");
    // Fallback: try wl-paste for Wayland
    match std::process::Command::new("wl-paste")
        .args(["--type", "image/png"])
        .output()
    {
        Ok(output) => {
            tracing::info!("wl-paste: status={}, stdout={} bytes, stderr={}",
                output.status, output.stdout.len(), String::from_utf8_lossy(&output.stderr));
            if output.status.success() && !output.stdout.is_empty() {
                return Some(output.stdout);
            }
        }
        Err(e) => {
            tracing::info!("wl-paste failed to run: {}", e);
        }
    }

    tracing::info!("get_clipboard_image_png: no image found");
    None
}

/// Encode RGBA pixel data to PNG bytes in memory.
fn encode_rgba_to_png(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    let img: ::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>> =
        ::image::ImageBuffer::from_raw(width, height, rgba.to_vec())?;
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, ::image::ImageFormat::Png).ok()?;
    Some(buf.into_inner())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn truncate_filename(name: &str, max: usize) -> String {
    if name.len() <= max {
        name.to_string()
    } else {
        format!("{}...", &name[..max])
    }
}

fn mime_from_extension(filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "json" => "application/json",
        "zip" => "application/zip",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        _ => "application/octet-stream",
    }
    .to_string()
}
