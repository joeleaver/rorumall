use rinch::prelude::*;
use crate::stores::{get_groups_store, get_members_store};
use crate::components::ui::role_editor::role_editor;

#[component]
pub fn group_settings(host: String, group_id: String) -> NodeHandle {
    let active_tab = use_signal(|| "overview".to_string());

    // Get group name from joined_groups (current_group may not be set)
    let group_name = get_groups_store()
        .joined_groups
        .get()
        .iter()
        .find(|g| g.group_id == group_id)
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "Group".to_string());

    let group_name_val = use_signal(|| group_name.clone());

    let host_sig = use_signal(|| host);
    let gid_sig = use_signal(|| group_id);
    let members_store = get_members_store();

    rsx! {
        Stack {
            gap: "md",

            Title {
                order: 4,
                {format!("{} Settings", group_name)}
            }

            // Tab bar
            Tabs {
                value: {active_tab.get().clone()},

                TabsList {
                    Tab { value: "overview", onclick: move || active_tab.set("overview".to_string()), "Overview" }
                    Tab { value: "members", onclick: move || active_tab.set("members".to_string()), "Members" }
                    Tab { value: "roles", onclick: move || active_tab.set("roles".to_string()), "Roles" }
                }
            }

            // Manual tab panel switching (TabsPanel doesn't hide inactive panels)
            if active_tab.get().as_str() == "overview" {
                Stack {
                    gap: "md",
                    p: "md",

                    TextInput {
                        label: "Group Name",
                        value_fn: move || group_name_val.get().clone(),
                        disabled: true,
                    }
                }
            } else if active_tab.get().as_str() == "members" {
                Stack {
                    gap: "xs",
                    p: "md",

                    for member in members_store.members.get().get(&gid_sig.get()).cloned().unwrap_or_default() {
                        Group {
                            gap: "sm",
                            style: "padding: 8px 0; border-bottom: 1px solid var(--rinch-color-dark-4, #373a40);",

                            Avatar {
                                size: "sm",
                                color: "indigo",
                                {member.user_id.chars().next().unwrap_or('?').to_uppercase().to_string()}
                            }

                            Stack {
                                gap: "0",

                                Text {
                                    size: "sm",
                                    weight: "600",
                                    {member.user_id.split('@').next().unwrap_or(&member.user_id).to_string()}
                                }

                                Text {
                                    size: "xs",
                                    color: "dimmed",
                                    {member.roles.first().cloned().unwrap_or_else(|| "member".to_string())}
                                }
                            }
                        }
                    }
                }
            } else if active_tab.get().as_str() == "roles" {
                Stack {
                    p: "md",
                    {role_editor(__scope, host_sig.get().clone(), gid_sig.get().clone())}
                }
            }
        }
    }
}
