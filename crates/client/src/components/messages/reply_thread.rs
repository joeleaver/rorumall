use rinch::prelude::*;
use crate::stores::{get_messages_store, StoredMessage};

#[component]
pub fn reply_thread(parent_id: String, channel_id: String, group_id: String) -> NodeHandle {
    let messages_store = get_messages_store();

    let replies = use_signal(|| -> Vec<StoredMessage> {
        messages_store
            .messages
            .get()
            .get(&channel_id)
            .map(|ch| {
                ch.messages
                    .iter()
                    .filter(|m| m.parent_id.as_deref() == Some(&parent_id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    });

    if replies.get().is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        div {
            style: "margin-left: 40px; padding-left: 12px; border-left: 2px solid var(--rinch-color-dark-4, #373a40);",

            for reply in replies.get().clone() {
                {crate::components::messages::message_item::message_item(__scope, reply, group_id.clone())}
            }
        }
    }
}
