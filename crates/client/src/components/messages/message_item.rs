use rinch::prelude::*;
use crate::stores::{get_members_store, StoredMessage};
use rorumall_shared::MessageType;

#[component]
pub fn message_item(msg: StoredMessage, group_id: String) -> NodeHandle {
    match msg.message_type {
        MessageType::Article => {
            crate::components::messages::article_item::article_item(__scope, msg, group_id)
        }
        MessageType::Memo => {
            crate::components::messages::memo_item::memo_item(__scope, msg, group_id)
        }
        MessageType::Message => {
            chat_message(__scope, msg, group_id)
        }
    }
}

#[component]
fn chat_message(msg: StoredMessage, group_id: String) -> NodeHandle {
    let user_display = msg.user_id.split('@').next().unwrap_or(&msg.user_id).to_string();
    let time = msg.created_at.format("%H:%M").to_string();
    let has_attachments = !msg.attachments.is_empty();
    let parent_id = use_signal(|| msg.parent_id.clone());
    let attachments = use_signal(|| msg.attachments.clone());
    let content = use_signal(|| msg.content.clone());

    let avatar_url = {
        let members = get_members_store()
            .get_group_members(&group_id)
            .unwrap_or_default();
        members.iter()
            .find(|m| m.user_id == msg.user_id)
            .and_then(|m| m.avatar.clone())
            .unwrap_or_default()
    };

    rsx! {
        div {
            style: "display: flex; gap: 12px; padding: 8px 0; align-items: flex-start;",

            Avatar {
                size: "sm",
                color: "indigo",
                name: user_display.clone(),
                src: avatar_url,
            }

            div {
                style: "flex: 1; min-width: 0;",

                Group {
                    gap: "xs",

                    Text {
                        size: "sm",
                        weight: "600",
                        {user_display}
                    }

                    Text {
                        size: "xs",
                        color: "dimmed",
                        {time}
                    }
                }

                // Reply indicator
                if parent_id.get().is_some() {
                    Text {
                        size: "xs",
                        color: "dimmed",
                        style: "margin-bottom: 4px;",
                        {|| { let p = parent_id.get().clone().unwrap_or_default(); format!("Replying to {}", &p[..8.min(p.len())]) }}
                    }
                }

                Text {
                    size: "sm",
                    {content.get().clone()}
                }

                // Attachments
                if has_attachments {
                    div {
                        style: "margin-top: 8px; display: flex; flex-wrap: wrap; gap: 8px;",
                        for att in attachments.get().clone() {
                            {crate::components::ui::attachment_display::attachment_display(__scope, att)}
                        }
                    }
                }
            }
        }
    }
}
