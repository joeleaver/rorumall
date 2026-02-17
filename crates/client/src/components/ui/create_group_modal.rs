use rinch::prelude::*;
use crate::stores::{get_auth_store, get_groups_store};

#[component]
pub fn create_group_modal(opened: Signal<bool>) -> NodeHandle {
    let name = use_signal(|| String::new());
    let description = use_signal(|| String::new());
    let creating = use_signal(|| false);
    let error = use_signal(|| Option::<String>::None);

    let on_create = move || {
        let n = name.get().clone();
        if n.trim().is_empty() {
            return;
        }
        creating.set(true);
        error.set(None);
        let desc = description.get().clone();
        let client = get_auth_store().make_client();
        crate::runtime::spawn(
            async move {
                let desc_opt = if desc.is_empty() {
                    None
                } else {
                    Some(desc.as_str())
                };
                client.create_group(&n, desc_opt).await
            },
            move |result| {
                match result {
                    Ok(group) => {
                        let joined = rorumall_shared::UserJoinedGroup {
                            group_id: group.id.clone(),
                            host: None,
                            name: group.name.clone(),
                            avatar: group.avatar.clone(),
                            joined_at: group.created_at.clone(),
                        };
                        get_groups_store()
                            .joined_groups
                            .update(|groups| groups.push(joined));
                        name.set(String::new());
                        description.set(String::new());
                        opened.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                    }
                }
                creating.set(false);
            },
        );
    };

    rsx! {
        Modal {
            opened_fn: move || opened.get(),
            onclose: move || opened.set(false),
            title: "Create Group",

            Stack {
                gap: "md",

                if error.get().is_some() {
                    Alert {
                        color: "red",
                        {error.get().clone().unwrap_or_default()}
                    }
                }

                TextInput {
                    label: "Group Name",
                    placeholder: "My awesome group",
                    value_fn: move || name.get().clone(),
                    oninput: move |val: String| name.set(val),
                }

                Textarea {
                    label: "Description",
                    placeholder: "What's this group about?",
                    value_fn: move || description.get().clone(),
                    oninput: move |val: String| description.set(val),
                }

                Group {
                    justify: "flex-end",

                    Button {
                        variant: "subtle",
                        onclick: move || opened.set(false),
                        "Cancel"
                    }

                    Button {
                        variant: "filled",
                        color: "indigo",
                        loading: creating.get(),
                        onclick: move || on_create(),
                        "Create"
                    }
                }
            }
        }
    }
}
