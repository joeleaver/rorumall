use rinch::prelude::*;
use crate::components::profile::presence_indicator::presence_indicator;

#[component]
pub fn profile_card(
    user_id: String,
    display_name: String,
    bio: String,
    avatar_url: String,
    status: String,
) -> NodeHandle {
    let bio_sig = use_signal(|| bio);
    let user_id_sig = use_signal(|| user_id);
    let display_name_sig = use_signal(|| display_name);
    let avatar_url_sig = use_signal(|| avatar_url);

    rsx! {
        Card {
            shadow: "sm",
            p: "lg",

            Group {
                gap: "md",

                div {
                    style: "position: relative;",

                    Avatar {
                        size: "lg",
                        color: "indigo",
                        radius: "xl",
                        src: {avatar_url_sig.get().clone()},
                        name: display_name_sig.get().clone(),
                    }

                    div {
                        style: "position: absolute; bottom: 0; right: 0;",
                        {presence_indicator(__scope, status, "sm".to_string())}
                    }
                }

                Stack {
                    gap: "xs",

                    Text {
                        weight: "700",
                        {display_name_sig.get().clone()}
                    }

                    Text {
                        size: "xs",
                        color: "dimmed",
                        {user_id_sig.get().clone()}
                    }

                    if !bio_sig.get().is_empty() {
                        Text {
                            size: "sm",
                            {bio_sig.get().clone()}
                        }
                    }
                }
            }
        }
    }
}
