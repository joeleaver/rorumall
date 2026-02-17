use rinch::prelude::*;
use crate::stores::get_auth_store;

#[component]
pub fn file_picker(attachments: Signal<Vec<rorumall_shared::Attachment>>) -> NodeHandle {
    let uploading = use_signal(|| false);

    let on_pick = move || {
        uploading.set(true);
        let client = get_auth_store().make_client();
        crate::runtime::spawn(
            async move {
                let path = rinch::dialogs::open_file()
                    .add_filter("All files", &["*"])
                    .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                    .pick_file();
                let path = path?;
                let data = match std::fs::read(&path) {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::error!("Failed to read file: {}", e);
                        return None;
                    }
                };
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "file".to_string());
                let mime = mime_from_ext(&filename);
                match client.upload_file(data, &filename, &mime).await {
                    Ok(attachment) => Some(attachment),
                    Err(e) => {
                        tracing::error!("Upload failed: {}", e);
                        None
                    }
                }
            },
            move |result: Option<rorumall_shared::Attachment>| {
                if let Some(attachment) = result {
                    attachments.update(|v| v.push(attachment));
                }
                uploading.set(false);
            },
        );
    };

    rsx! {
        Button {
            variant: "subtle",
            size: "sm",
            loading: uploading.get(),
            onclick: move || on_pick(),
            "Upload File"
        }
    }
}

fn mime_from_ext(filename: &str) -> String {
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
        _ => "application/octet-stream",
    }
    .to_string()
}
