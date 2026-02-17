use rinch::prelude::*;
use crate::stores::{get_auth_store, get_profile_store};

#[component]
pub fn profile_edit_form() -> NodeHandle {
    let profile_store = get_profile_store();
    let current = profile_store.current.get().clone();
    let current2 = current.clone();

    let display_name = use_signal(move || {
        current
            .as_ref()
            .and_then(|p| p.display_name.clone())
            .unwrap_or_default()
    });
    let bio = use_signal(move || {
        current2
            .as_ref()
            .and_then(|p| p.bio.clone())
            .unwrap_or_default()
    });
    let saving = use_signal(|| false);
    let error = use_signal(|| Option::<String>::None);

    let on_save = move || {
        saving.set(true);
        error.set(None);
        let client = get_auth_store().make_client();
        let name = display_name.get().clone();
        let b = bio.get().clone();
        crate::runtime::spawn(
            async move {
                let req = rorumall_shared::UpdateProfileRequest {
                    display_name: if name.is_empty() { None } else { Some(name) },
                    bio: if b.is_empty() { None } else { Some(b) },
                    avatar: None,
                    metadata: None,
                };
                client.update_profile(&req).await.map_err(|e| e.to_string())
            },
            move |result| {
                match result {
                    Ok(profile) => {
                        get_profile_store().current.set(Some(profile));
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
                saving.set(false);
            },
        );
    };

    rsx! {
        Stack {
            gap: "md",

            Title {
                order: 5,
                "Edit Profile"
            }

            if error.get().is_some() {
                Alert {
                    color: "red",
                    {error.get().clone().unwrap_or_default()}
                }
            }

            TextInput {
                label: "Display Name",
                placeholder: "Your display name",
                value_fn: move || display_name.get().clone(),
                oninput: move |val: String| display_name.set(val),
            }

            Textarea {
                label: "Bio",
                placeholder: "Tell us about yourself...",
                value_fn: move || bio.get().clone(),
                oninput: move |val: String| bio.set(val),
            }

            Button {
                variant: "filled",
                color: "indigo",
                loading: saving.get(),
                onclick: move || on_save(),
                "Save Profile"
            }
        }
    }
}
