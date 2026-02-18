use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};
use crate::navigation::{navigate, AppRoute};
use crate::stores::{get_auth_store, get_profile_store, get_presence_store};

#[component]
pub fn profile_view() -> NodeHandle {
    let auth = get_auth_store();
    let profile_store = get_profile_store();
    let _presence_store = get_presence_store();
    let loading = use_signal(|| true);
    let editing = use_signal(|| false);

    let display_name = use_signal(|| String::new());
    let bio = use_signal(|| String::new());
    let error_msg = use_signal(|| None::<String>);
    let success_msg = use_signal(|| None::<String>);

    // Load profile
    use_mount(move || {
        let client = auth.make_client();
        crate::runtime::spawn(
            async move {
                let profile_result = client.get_json::<rorumall_shared::UserProfile>("/api/me/profile").await;
                let presence_result = client.get_own_presence().await;
                (profile_result, presence_result)
            },
            move |(profile_result, presence_result)| {
                match profile_result {
                    Ok(profile) => {
                        display_name.set(profile.display_name.clone().unwrap_or_default());
                        bio.set(profile.bio.clone().unwrap_or_default());
                        get_profile_store().set_current(profile);
                    }
                    Err(e) => tracing::error!("Failed to load profile: {}", e),
                }

                match presence_result {
                    Ok(presence) => get_presence_store().set_current(presence),
                    Err(e) => tracing::error!("Failed to load presence: {}", e),
                }

                loading.set(false);
            },
        );
        || {}
    });

    let on_save = move || {
        let client = auth.make_client();
        let dn = display_name.get().clone();
        let b = bio.get().clone();

        crate::runtime::spawn(
            async move {
                let update = rorumall_shared::UpdateProfileRequest {
                    display_name: Some(dn),
                    bio: Some(b),
                    ..Default::default()
                };
                client.update_profile(&update).await.map_err(|e| format!("Failed to update: {}", e))
            },
            move |result| {
                match result {
                    Ok(profile) => {
                        get_profile_store().set_current(profile);
                        editing.set(false);
                        success_msg.set(Some("Profile updated!".to_string()));
                    }
                    Err(e) => {
                        error_msg.set(Some(e));
                    }
                }
            },
        );
    };

    let on_logout = move || {
        auth.clear_session();
        crate::ws::clear_connections();
        navigate(AppRoute::Login);
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; padding: 24px;",

            // Header
            Group {
                justify: "space-between",
                style: "margin-bottom: 24px;",

                Group {
                    gap: "sm",

                    ActionIcon {
                        variant: "subtle",
                        onclick: move || navigate(AppRoute::Home),
                        {render_tabler_icon(__scope, TablerIcon::ArrowLeft, TablerIconStyle::Outline)}
                    }

                    Title {
                        order: 3,
                        "Profile"
                    }
                }

                Button {
                    variant: "light",
                    color: "red",
                    onclick: move || on_logout(),
                    "Sign Out"
                }
            }

            if loading.get() {
                Stack {
                    align: "center",
                    p: "xl",
                    Loader {}
                }
            }

            if !loading.get() {
                Stack {
                    gap: "lg",
                    style: "max-width: 600px;",

                    // Avatar section
                    Group {
                        gap: "lg",

                        Avatar {
                            size: "xl",
                            color: "indigo",
                            src: {profile_store.current.get().as_ref().and_then(|p| p.avatar.clone()).unwrap_or_default()},
                            name: {profile_store.current.get().as_ref().and_then(|p| p.display_name.clone()).unwrap_or_else(|| auth.handle().unwrap_or_default())},
                        }

                        Stack {
                            gap: "xs",

                            Title {
                                order: 4,
                                {|| profile_store.current.get().as_ref().and_then(|p| p.display_name.clone()).unwrap_or_else(|| auth.handle().unwrap_or_default())}
                            }

                            Text {
                                color: "dimmed",
                                size: "sm",
                                {|| format!("@{}", auth.handle().unwrap_or_default())}
                            }
                        }
                    }

                    Divider {}

                    // Presence selector
                    div {
                        {crate::components::profile::presence_selector::presence_selector(__scope)}
                    }

                    Divider {}

                    // Profile edit form
                    if editing.get() {
                        Stack {
                            gap: "md",

                            TextInput {
                                label: "Display Name",
                                value_fn: move || display_name.get().clone(),
                                oninput: move |val: String| display_name.set(val),
                            }

                            Textarea {
                                label: "Bio",
                                value_fn: move || bio.get().clone(),
                                oninput: move |val: String| bio.set(val),
                            }

                            if let Some(_err) = error_msg.get().clone() {
                                Alert {
                                    color: "red",
                                    variant: "light",
                                    {error_msg.get().clone().unwrap_or_default()}
                                }
                            }

                            Group {
                                gap: "sm",

                                Button {
                                    variant: "filled",
                                    color: "indigo",
                                    onclick: move || on_save(),
                                    "Save"
                                }

                                Button {
                                    variant: "light",
                                    onclick: move || editing.set(false),
                                    "Cancel"
                                }
                            }
                        }
                    }

                    if !editing.get() {
                        Stack {
                            gap: "md",

                            if profile_store.current.get().as_ref().and_then(|p| p.bio.clone()).is_some() {
                                Text { {profile_store.current.get().as_ref().and_then(|p| p.bio.clone()).unwrap_or_default()} }
                            }

                            Button {
                                variant: "light",
                                onclick: move || editing.set(true),
                                "Edit Profile"
                            }
                        }
                    }

                    if let Some(_msg) = success_msg.get().clone() {
                        Alert {
                            color: "green",
                            variant: "light",
                            {success_msg.get().clone().unwrap_or_default()}
                        }
                    }

                    Divider {}

                    // Privacy settings
                    div {
                        {crate::components::profile::privacy_settings::privacy_settings(
                            __scope,
                            "public".to_string(),
                            "public".to_string(),
                            "public".to_string(),
                        )}
                    }
                }
            }
        }
    }
}
