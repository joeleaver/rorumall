use rinch::prelude::*;
use crate::navigation::{navigate, AppRoute};
use crate::stores::get_groups_store;

#[component]
pub fn group_list() -> NodeHandle {
    let groups_store = get_groups_store();
    let groups = use_signal(|| groups_store.joined_groups.get().clone());
    let current = use_signal(|| groups_store.current_group.get().clone());

    rsx! {
        Stack {
            gap: "xs",
            p: "xs",

            for group in groups.get().clone() {
                Tooltip {
                    label: {group.name.clone()},
                    position: "right",

                    ActionIcon {
                        variant: {|| if current.get().as_ref().map(|g| &g.id) == Some(&group.group_id) { "filled".to_string() } else { "subtle".to_string() }},
                        color: "indigo",
                        size: "xl",
                        radius: "md",
                        onclick: {
                            let host = group.host.clone().unwrap_or_default();
                            let gid = group.group_id.clone();
                            move || {
                                navigate(AppRoute::Group {
                                    host: host.clone(),
                                    group_id: gid.clone(),
                                });
                            }
                        },

                        Avatar {
                            size: "sm",
                            color: "indigo",
                            radius: "md",
                            src: {group.avatar.clone().unwrap_or_default()},
                            {group.name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                        }
                    }
                }
            }
        }
    }
}
