use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};
use crate::navigation::{navigate, AppRoute};
use crate::stores::get_auth_store;

#[component]
pub fn message_input(channel_id: String, group_id: String, host: String) -> NodeHandle {
    let input_text = use_signal(|| String::new());
    let message_type = use_signal(|| "message".to_string());
    let reply_to = use_signal(|| None::<String>);
    let pending_attachments = use_signal(|| Vec::<rorumall_shared::Attachment>::new());

    let cid = use_signal(|| channel_id.clone());
    let h = use_signal(|| host.clone());

    let on_send = move || {
        let text = input_text.get().clone();
        if text.trim().is_empty() {
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

            let attachments = pending_attachments.get().clone();
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
                    pending_attachments.set(Vec::new());
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
        let client = get_auth_store().make_client();
        let path = rinch::dialogs::open_file()
            .add_filter("All files", &["*"])
            .pick_file();
        if let Some(path) = path {
            crate::runtime::spawn(
                async move {
                    let filename = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "file".to_string());
                    let data = std::fs::read(&path)
                        .map_err(|e| rorumall_shared::ApiError::Network(e.to_string()))?;
                    let mime = mime_from_extension(&filename);
                    client.upload_file(data, &filename, &mime).await
                },
                move |result| match result {
                    Ok(attachment) => {
                        pending_attachments.update(|atts| atts.push(attachment));
                    }
                    Err(e) => {
                        tracing::error!("Upload failed: {}", e);
                    }
                },
            );
        }
    };

    rsx! {
        div {
            class: "message-input-area",
            style: "border-top: 1px solid var(--rinch-color-dark-4, #373a40); padding: 12px 16px;",

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

            // Pending attachments
            if !pending_attachments.get().is_empty() {
                Group {
                    gap: "xs",
                    style: "margin-bottom: 8px;",

                    for att in pending_attachments.get().clone() {
                        Badge {
                            variant: "light",
                            {att.id.clone()}
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
                    onclick: move || {
                        let text = input_text.get().clone();
                        if text.trim().is_empty() { return; }
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
                            let attachments = pending_attachments.get().clone();
                            let parent = reply_to.get().clone();
                            let result = if let Some(pid) = parent {
                                handle.send_reply(&cid_val, &text, &nonce, &pid, mt, attachments)
                            } else {
                                handle.send_message_with_options(&cid_val, &text, &nonce, None, mt, attachments)
                            };
                            if result.is_ok() {
                                input_text.set(String::new());
                                reply_to.set(None);
                                pending_attachments.set(Vec::new());
                            }
                        }
                    },
                    {render_tabler_icon(__scope, TablerIcon::Send, TablerIconStyle::Outline)}
                }
            }
        }
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
