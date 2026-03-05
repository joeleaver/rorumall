use rinch::prelude::*;
use crate::stores::get_auth_store;

#[component]
pub fn privacy_settings(
    presence_policy: String,
    profile_policy: String,
    message_policy: String,
) -> NodeHandle {
    let pres = Signal::new(presence_policy);
    let prof = Signal::new(profile_policy);
    let msg = Signal::new(message_policy);
    let saving = Signal::new(false);

    let on_save = move || {
        saving.set(true);
        let client = get_auth_store().make_client();
        let presence = pres.get().clone();
        let profile = prof.get().clone();
        let message = msg.get().clone();
        crate::runtime::spawn(
            async move {
                let settings = rorumall_shared::PrivacySettings {
                    presence_visibility: serde_json::from_str(&format!("\"{}\"", presence)).unwrap_or_default(),
                    profile_visibility: serde_json::from_str(&format!("\"{}\"", profile)).unwrap_or_default(),
                    membership_visibility: serde_json::from_str(&format!("\"{}\"", message)).unwrap_or_default(),
                };
                match client
                    .update_privacy_settings(&settings)
                    .await
                {
                    Ok(_) => {
                        tracing::info!("Privacy settings updated");
                    }
                    Err(e) => {
                        tracing::error!("Failed to update privacy: {}", e);
                    }
                }
            },
            move |()| {
                saving.set(false);
            },
        );
    };

    rsx! {
        Stack {
            gap: "md",

            Title {
                order: 5,
                "Privacy Settings"
            }

            Stack {
                gap: "sm",

                Select {
                    label: "Who can see my presence",
                    value_fn: move || pres.get().clone(),
                    onchange: move |val: String| pres.set(val),
                    option { value: "public", "public" }
                    option { value: "contacts", "contacts" }
                    option { value: "nobody", "nobody" }
                }

                Select {
                    label: "Who can see my profile",
                    value_fn: move || prof.get().clone(),
                    onchange: move |val: String| prof.set(val),
                    option { value: "public", "public" }
                    option { value: "contacts", "contacts" }
                    option { value: "nobody", "nobody" }
                }

                Select {
                    label: "Who can message me",
                    value_fn: move || msg.get().clone(),
                    onchange: move |val: String| msg.set(val),
                    option { value: "public", "public" }
                    option { value: "contacts", "contacts" }
                    option { value: "nobody", "nobody" }
                }
            }

            Button {
                variant: "filled",
                color: "indigo",
                loading: saving.get(),
                onclick: move || on_save(),
                "Save Privacy Settings"
            }
        }
    }
}
