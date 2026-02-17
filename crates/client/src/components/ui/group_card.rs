use rinch::prelude::*;

#[component]
pub fn group_card(group: rorumall_shared::Group) -> NodeHandle {
    let avatar = use_signal(|| group.avatar.clone().unwrap_or_default());
    let initial = use_signal(|| group.name.chars().next().unwrap_or('?').to_uppercase().to_string());
    let name = use_signal(|| group.name.clone());
    let description = use_signal(|| group.description.clone());

    rsx! {
        Card {
            shadow: "sm",
            p: "md",

            Group {
                gap: "md",

                Avatar {
                    size: "lg",
                    color: "indigo",
                    radius: "md",
                    src: {avatar.get().clone()},
                    {initial.get().clone()}
                }

                Stack {
                    gap: "xs",

                    Text {
                        weight: "700",
                        {name.get().clone()}
                    }

                    if description.get().is_some() {
                        Text {
                            size: "sm",
                            color: "dimmed",
                            {description.get().clone().unwrap_or_default()}
                        }
                    }
                }
            }
        }
    }
}
