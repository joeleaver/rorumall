use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};
use crate::navigation::{get_nav, navigate, AppRoute};
use crate::stores::get_auth_store;

#[component]
pub fn article_editor_view() -> NodeHandle {
    let nav = get_nav();
    let (host, group_id, channel_id) = match nav.get().clone() {
        AppRoute::ComposeArticle { host, group_id, channel_id } => (host, group_id, channel_id),
        _ => return rsx! { div { "Invalid route" } },
    };

    let title = use_signal(|| String::new());
    let body = use_signal(|| String::new());
    let error_msg = use_signal(|| None::<String>);
    let loading = use_signal(|| false);

    let host_c = host.clone();
    let gid_c = group_id.clone();
    let cid_c = channel_id.clone();

    let on_publish = move || {
        let t = title.get().clone();
        let b = body.get().clone();

        if t.is_empty() || b.is_empty() {
            error_msg.set(Some("Title and body are required".to_string()));
            return;
        }

        loading.set(true);
        error_msg.set(None);

        let domain = get_auth_store().domain();
        let ws_host = if host_c.is_empty() { domain } else { host_c.clone() };

        if let Some(handle) = crate::ws::get_handle(&ws_host) {
            let nonce = uuid::Uuid::new_v4().to_string();
            match handle.send_message_with_options(
                &cid_c,
                &b,
                &nonce,
                Some(t),
                Some(rorumall_shared::MessageType::Article),
                vec![],
            ) {
                Ok(()) => {
                    navigate(AppRoute::Channel {
                        host: host_c.clone(),
                        group_id: gid_c.clone(),
                        channel_id: cid_c.clone(),
                    });
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to send: {}", e)));
                }
            }
        } else {
            error_msg.set(Some("Not connected to server".to_string()));
        }
        loading.set(false);
    };

    let host_back = host.clone();
    let gid_back = group_id.clone();
    let cid_back = channel_id.clone();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; padding: 24px;",

            // Header
            Group {
                justify: "space-between",
                style: "margin-bottom: 16px;",

                Group {
                    gap: "sm",

                    ActionIcon {
                        variant: "subtle",
                        onclick: move || navigate(AppRoute::Channel {
                            host: host_back.clone(),
                            group_id: gid_back.clone(),
                            channel_id: cid_back.clone(),
                        }),
                        {render_tabler_icon(__scope, TablerIcon::ArrowLeft, TablerIconStyle::Outline)}
                    }

                    Title {
                        order: 3,
                        "Compose Article"
                    }
                }

                Button {
                    variant: "filled",
                    color: "indigo",
                    disabled: loading.get(),
                    onclick: move || on_publish(),
                    {render_tabler_icon(__scope, TablerIcon::Send, TablerIconStyle::Outline)}
                    " Publish"
                }
            }

            if error_msg.get().is_some() {
                Alert {
                    color: "red",
                    variant: "light",
                    {error_msg.get().clone().unwrap_or_default()}
                }
            }

            Stack {
                gap: "md",
                style: "flex: 1;",

                TextInput {
                    label: "Title",
                    placeholder: "Article title",
                    size: "lg",
                    value_fn: move || title.get().clone(),
                    oninput: move |val: String| title.set(val),
                }

                Textarea {
                    label: "Content (Markdown)",
                    placeholder: "Write your article in Markdown...",
                    style: "flex: 1;",
                    min_rows: 15,
                    value_fn: move || body.get().clone(),
                    oninput: move |val: String| body.set(val),
                }

                // Preview
                if !body.get().is_empty() {
                    div {
                        style: "border: 1px solid var(--rinch-color-dark-4, #373a40); border-radius: var(--rinch-radius-md); padding: 16px;",

                        Text {
                            size: "sm",
                            color: "dimmed",
                            style: "margin-bottom: 8px;",
                            "Preview"
                        }

                        div {
                            class: "markdown-content",
                            // Render markdown preview
                            {crate::components::messages::article_item::render_markdown(&body.get())}
                        }
                    }
                }
            }
        }
    }
}
