use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};
use rorumall_shared::Attachment;

use crate::stores::get_auth_store;

#[component]
pub fn attachment_display(att: Attachment) -> NodeHandle {
    let is_image = att.mime.starts_with("image/");

    let filename = use_signal(|| att.id.clone());
    let size_label = use_signal(|| format_size(att.size));
    let url = use_signal(|| att.url.clone());
    let download_status = use_signal(|| DownloadStatus::Idle);

    let on_download = move || {
        download_status.set(DownloadStatus::Downloading);
        let client = get_auth_store().make_client();
        let download_url = url.get().clone();
        let default_name = filename.get().clone();
        crate::runtime::spawn(
            async move {
                let download_dir = dirs::download_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_default());
                let path = download_dir.join(&default_name);
                match client.get_bytes(&download_url).await {
                    Ok(bytes) => {
                        if let Err(e) = std::fs::write(&path, &bytes) {
                            tracing::error!("Failed to write file: {}", e);
                            return false;
                        }
                        tracing::info!("Saved attachment to {}", path.display());
                        true
                    }
                    Err(e) => {
                        tracing::error!("Download failed: {}", e);
                        false
                    }
                }
            },
            move |ok| {
                if ok {
                    download_status.set(DownloadStatus::Done);
                } else {
                    download_status.set(DownloadStatus::Idle);
                }
            },
        );
    };

    if is_image {
        rsx! {
            Card {
                shadow: "xs",
                p: "xs",
                style: "max-width: 300px;",

                Stack {
                    gap: "xs",

                    img {
                        src: {url.get().clone()},
                        style: "max-width: 100%; border-radius: var(--rinch-radius-sm, 4px);",
                        alt: {filename.get().clone()},
                    }

                    Text {
                        size: "xs",
                        color: "dimmed",
                        {format!("{} {}", filename.get(), size_label.get())}
                    }
                }
            }
        }
    } else {
        rsx! {
            Card {
                shadow: "xs",
                p: "sm",

                Group {
                    gap: "sm",

                    {render_tabler_icon(__scope, TablerIcon::File, TablerIconStyle::Outline)}

                    Stack {
                        gap: "0",

                        Text {
                            size: "sm",
                            {filename.get().clone()}
                        }

                        if !size_label.get().is_empty() {
                            Text {
                                size: "xs",
                                color: "dimmed",
                                {size_label.get().clone()}
                            }
                        }
                    }

                    if download_status.get() == DownloadStatus::Idle {
                        ActionIcon {
                            variant: "subtle",
                            onclick: move || on_download(),
                            {render_tabler_icon(__scope, TablerIcon::Download, TablerIconStyle::Outline)}
                        }
                    }

                    if download_status.get() == DownloadStatus::Downloading {
                        Loader {
                            size: "xs",
                        }
                    }

                    if download_status.get() == DownloadStatus::Done {
                        ActionIcon {
                            variant: "subtle",
                            color: "green",
                            onclick: move || download_status.set(DownloadStatus::Idle),
                            {render_tabler_icon(__scope, TablerIcon::Check, TablerIconStyle::Outline)}
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum DownloadStatus {
    Idle,
    Downloading,
    Done,
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
