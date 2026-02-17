use rinch::prelude::*;
use crate::stores::StoredMessage;

#[component]
pub fn memo_item(msg: StoredMessage) -> NodeHandle {
    let user_display = msg.user_id.split('@').next().unwrap_or(&msg.user_id).to_string();
    let time = msg.created_at.format("%H:%M").to_string();

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
