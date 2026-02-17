use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};
use crate::navigation::{get_nav, AppRoute};
use crate::stores::{get_auth_store, get_groups_store, get_messages_store, StoredMessage};

#[component]
pub fn channel_view() -> NodeHandle {
    let nav = get_nav();
    let (host, group_id, channel_id) = match nav.get().clone() {
        AppRoute::Channel { host, group_id, channel_id } => (host, group_id, channel_id),
        _ => return rsx! { div { "No channel selected" } },
    };

    let messages_store = get_messages_store();
    let auth = get_auth_store();
    let loading = use_signal(|| false);

    // Load channel messages
    let ch_id = channel_id.clone();
    let g_id = group_id.clone();
    use_effect(move || {
        if !messages_store.is_channel_loaded(&ch_id) {
            loading.set(true);
            let client = auth.make_client();
            let ch = ch_id.clone();
            let gid = g_id.clone();

            crate::runtime::spawn(
                async move {
                    let path = format!("/api/groups/{}/channels/{}/messages", gid, ch);
                    let result = client.get_json::<rorumall_shared::MessagesPage>(&path).await;
                    (ch, result)
                },
                move |(ch, result)| {
                    match result {
                        Ok(page) => {
                            let stored: Vec<StoredMessage> = page.items.into_iter().map(|m| {
                                StoredMessage {
                                    id: m.id,
                                    user_id: m.sender_user_id,
                                    title: m.title,
                                    content: m.body,
                                    message_type: m.message_type.unwrap_or(rorumall_shared::MessageType::Message),
                                    created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at)
                                        .map(|dt| dt.with_timezone(&chrono::Utc))
                                        .unwrap_or_else(|_| chrono::Utc::now()),
                                    parent_id: m.parent_id,
                                    parent_message_type: m.parent_message_type,
                                    attachments: m.attachments,
                                }
                            }).collect();
                            get_messages_store().set_channel_history(&ch, stored);
                        }
                        Err(e) => {
                            tracing::error!("Failed to load messages: {}", e);
                        }
                    }
                    loading.set(false);
                },
            );
        }
    }, channel_id.clone());

    // Subscribe via WS
    let ch_for_ws = channel_id.clone();
    let host_for_ws = host.clone();
    use_mount(move || {
        let domain = get_auth_store().domain();
        let ws_host = if host_for_ws.is_empty() { domain } else { host_for_ws.clone() };
        if let Some(handle) = crate::ws::get_handle(&ws_host) {
            let _ = handle.subscribe(&ch_for_ws);
        }
        || {}
    });

    // Get channel name from groups store
    let channel_name = get_groups_store()
        .channels.get()
        .iter()
        .find(|c| c.id == channel_id)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| channel_id.clone());

    let cid = use_signal(|| channel_id.clone());

    rsx! {
        div {
            style: "display: flex; flex-direction: column; flex: 1; overflow: hidden;",

            // Channel header
            div {
                style: "padding: 12px 16px; border-bottom: 1px solid var(--rinch-color-dark-4, #373a40); display: flex; align-items: center; gap: 8px;",

                div {
                    {render_tabler_icon(__scope, TablerIcon::Hash, TablerIconStyle::Outline)}
                }

                Title {
                    order: 4,
                    {channel_name}
                }
            }

            // Message list
            div {
                class: "message-list",
                style: "flex: 1; overflow-y: auto; padding: 16px; min-height: 0;",

                if loading.get() {
                    Stack {
                        align: "center",
                        p: "xl",
                        Loader {}
                    }
                }

                for msg in messages_store.messages.get().get(&cid.get()).map(|ch| ch.messages.clone()).unwrap_or_default() {
                    div {
                        key: msg.id.clone(),
                        {crate::components::messages::message_item::message_item(__scope, msg)}
                    }
                }
            }

            // Message input
            div {
                {crate::components::messages::message_input::message_input(__scope, channel_id.clone(), group_id.clone(), host.clone())}
            }
        }
    }
}
