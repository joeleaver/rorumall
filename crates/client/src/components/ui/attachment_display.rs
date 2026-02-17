use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};
use rorumall_shared::Attachment;

#[component]
pub fn attachment_display(att: Attachment) -> NodeHandle {
    let is_image = att.mime.starts_with("image/");

    let filename = use_signal(|| att.id.clone());
    let size_label = use_signal(|| format_size(att.size));
    let url = use_signal(|| att.url.clone());

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

                        Anchor {
                            href: {url.get().clone()},
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
                }
            }
        }
    }
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
