use rinch::prelude::*;
use crate::stores::StoredMessage;
use rorumall_shared::MessageType;

#[component]
pub fn message_item(msg: StoredMessage) -> NodeHandle {
    match msg.message_type {
        MessageType::Article => {
            crate::components::messages::article_item::article_item(__scope, msg)
        }
        MessageType::Memo => {
            crate::components::messages::memo_item::memo_item(__scope, msg)
        }
        MessageType::Message => {
            chat_message(__scope, msg)
        }
    }
}

#[component]
fn chat_message(msg: StoredMessage) -> NodeHandle {
    let user_display = msg.user_id.split('@').next().unwrap_or(&msg.user_id).to_string();
    let time = msg.created_at.format("%H:%M").to_string();
    let has_attachments = !msg.attachments.is_empty();
    let parent_id = use_signal(|| msg.parent_id.clone());
    let attachments = use_signal(|| msg.attachments.clone());
    let content = use_signal(|| msg.content.clone());

    rsx! {
        div {
            style: "display: flex; gap: 12px; padding: 8px 0; align-items: flex-start;",

            Avatar {
                size: "sm",
                color: "indigo",
                {user_display.chars().next().unwrap_or('?').to_string()}
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
