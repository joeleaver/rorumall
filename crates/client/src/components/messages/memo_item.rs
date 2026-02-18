use rinch::prelude::*;
use crate::stores::{get_members_store, StoredMessage};

#[component]
pub fn memo_item(msg: StoredMessage, group_id: String) -> NodeHandle {
    let user_display = msg.user_id.split('@').next().unwrap_or(&msg.user_id).to_string();
    let time = msg.created_at.format("%H:%M").to_string();

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
        Card {
            shadow: "sm",
            p: "md",
            style: "margin: 8px 0; border-left: 3px solid var(--rinch-color-yellow-6, #fab005);",

            Stack {
                gap: "xs",

                Group {
                    justify: "space-between",

                    Group {
                        gap: "xs",

                        Badge {
                            variant: "light",
                            color: "yellow",
                            "Memo"
                        }

                        Avatar {
                            size: "xs",
                            color: "yellow",
                            name: user_display.clone(),
                            src: avatar_url,
                        }

                        Text {
                            size: "sm",
                            weight: "600",
                            {user_display}
                        }
                    }

                    Text {
                        size: "xs",
                        color: "dimmed",
                        {time}
                    }
                }

                Text {
                    size: "sm",
                    {msg.content.clone()}
                }
            }
        }
    }
}
