use rinch::prelude::*;

#[component]
pub fn group_card(group: rorumall_shared::Group) -> NodeHandle {
    let avatar = Signal::new(group.avatar.clone().unwrap_or_default());
    let name = Signal::new(group.name.clone());
    let description = Signal::new(group.description.clone());

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
                    name: name.get().clone(),
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
