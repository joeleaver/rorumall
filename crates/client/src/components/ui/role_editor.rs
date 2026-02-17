use rinch::prelude::*;
use crate::stores::{get_auth_store, get_members_store};

#[component]
pub fn role_editor(_host: String, group_id: String) -> NodeHandle {
    let gid_for_roles = group_id.clone();
    let roles = use_signal(move || {
        get_members_store()
            .group_roles
            .get()
            .get(&gid_for_roles)
            .cloned()
            .unwrap_or_default()
    });

    let new_role_name = use_signal(|| String::new());
    let creating = use_signal(|| false);

    let gid = group_id.clone();

    let on_create_role = move || {
        let name = new_role_name.get().clone();
        if name.trim().is_empty() {
            return;
        }
        creating.set(true);
        let client = get_auth_store().make_client();
        let gid = gid.clone();
        crate::runtime::spawn(
            async move {
                client
                    .create_role(&gid, &rorumall_shared::CreateRoleRequest {
                        name,
                        color: None,
                        position: None,
                    })
                    .await
                    .map(|role| (gid, role))
            },
            move |result| {
                match result {
                    Ok((gid, role)) => {
                        get_members_store().group_roles.update(|roles_map| {
                            roles_map.entry(gid).or_default().push(role);
                        });
                        new_role_name.set(String::new());
                    }
                    Err(e) => {
                        tracing::error!("Failed to create role: {}", e);
                    }
                }
                creating.set(false);
            },
        );
    };

    rsx! {
        Stack {
            gap: "md",

            Title {
                order: 5,
                "Roles"
            }

            for role in roles.get().clone() {
                Card {
                    shadow: "xs",
                    p: "sm",

                    Group {
                        justify: "space-between",

                        Group {
                            gap: "sm",

                            Badge {
                                variant: "filled",
                                color: {role.color.clone().unwrap_or_else(|| "indigo".to_string())},
                                {role.name.clone()}
                            }
                        }

                        Text {
                            size: "xs",
                            color: "dimmed",
                            {format!("Priority: {}", role.position)}
                        }
                    }
                }
            }

            Divider {}

            Group {
                gap: "sm",

                TextInput {
                    placeholder: "New role name",
                    style: "flex: 1;",
                    value_fn: move || new_role_name.get().clone(),
                    oninput: move |val: String| new_role_name.set(val),
                }

                Button {
                    variant: "filled",
                    color: "indigo",
                    loading: creating.get(),
                    onclick: move || on_create_role(),
                    "Add Role"
                }
            }
        }
    }
}
