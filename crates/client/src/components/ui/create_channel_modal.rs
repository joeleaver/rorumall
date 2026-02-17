use rinch::prelude::*;
use crate::stores::{get_auth_store, get_groups_store};

#[component]
pub fn create_channel_modal(
    opened: Signal<bool>,
    group_id: String,
) -> NodeHandle {
    let name = use_signal(|| String::new());
    let channel_type = use_signal(|| "text".to_string());
    let creating = use_signal(|| false);
    let error = use_signal(|| Option::<String>::None);

    let on_create = move || {
        let n = name.get().clone();
        if n.trim().is_empty() {
            return;
        }
        creating.set(true);
        error.set(None);
        let ct = channel_type.get().clone();
        let client = get_auth_store().make_client();
        let gid = group_id.clone();
        crate::runtime::spawn(
            async move {
                client.create_channel(&gid, &n, &ct).await
            },
            move |result| {
                match result {
                    Ok(channel) => {
                        get_groups_store()
                            .channels
                            .update(|chs| chs.push(channel));
                        name.set(String::new());
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
            title: "Create Channel",

            Stack {
                gap: "md",

                if error.get().is_some() {
                    Alert {
                        color: "red",
                        {error.get().clone().unwrap_or_default()}
                    }
                }

                TextInput {
                    label: "Channel Name",
                    placeholder: "general",
                    value_fn: move || name.get().clone(),
                    oninput: move |val: String| name.set(val),
                }

                Select {
                    label: "Type",
                    value_fn: move || channel_type.get().clone(),
                    onchange: move |val: String| channel_type.set(val),
                    option { value: "text", "text" }
                    option { value: "voice", "voice" }
                    option { value: "announcements", "announcements" }
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
