use rinch::prelude::*;
use crate::auth_session::AuthSession;
use crate::client_keys::{generate_keypair, save_keypair};
use crate::navigation::{navigate, AppRoute};
use crate::stores::get_auth_store;

#[component]
pub fn register_view() -> NodeHandle {
    let server_url = use_signal(|| get_auth_store().domain());
    let handle_input = use_signal(|| String::new());
    let password_input = use_signal(|| String::new());
    let confirm_password = use_signal(|| String::new());
    let error_msg = use_signal(|| None::<String>);
    let loading = use_signal(|| false);

    let on_register = move || {
        let server = server_url.get().clone();
        let handle = handle_input.get().clone();
        let password = password_input.get().clone();
        let confirm = confirm_password.get().clone();

        if handle.is_empty() || password.is_empty() {
            error_msg.set(Some("Please fill in all fields".to_string()));
            return;
        }

        if password != confirm {
            error_msg.set(Some("Passwords do not match".to_string()));
            return;
        }

        loading.set(true);
        error_msg.set(None);

        let auth = get_auth_store();
        auth.set_server_url(server.clone());
        let client = auth.make_client();

        crate::runtime::spawn(
            async move {
                let keys = generate_keypair();

                let register_req = rorumall_shared::RegisterRequest {
                    handle: handle.clone(),
                    password,
                    device_public_key: Some(keys.public_key.clone()),
                    device_name: Some("rorumall-desktop".to_string()),
                };

                match client
                    .post_json::<_, rorumall_shared::LoginResponse>(
                        &format!("{}/api/auth/register", crate::auth_session::api_base_url(&server)),
                        &register_req,
                    )
                    .await
                {
                    Ok(resp) => {
                        let mut keys = keys;
                        keys.key_id = resp.key_id;
                        save_keypair(&keys);

                        let session = AuthSession {
                            user_id: resp.user_id,
                            keys: Some(keys),
                        };
                        Ok((auth, session))
                    }
                    Err(e) => {
                        let msg = match &e {
                            rorumall_shared::ApiError::Http { body, .. } => {
                                rorumall_shared::try_problem_detail(body)
                                    .unwrap_or_else(|| e.to_string())
                            }
                            _ => e.to_string(),
                        };
                        Err(msg)
                    }
                }
            },
            move |result: Result<(_, AuthSession), String>| {
                loading.set(false);
                match result {
                    Ok((auth, session)) => {
                        auth.set_session(session);
                        navigate(AppRoute::Home);
                    }
                    Err(msg) => {
                        error_msg.set(Some(msg));
                    }
                }
            },
        );
    };

    rsx! {
        div {
            class: "auth-container",
            style: "display: flex; align-items: center; justify-content: center; min-height: 100vh; background: linear-gradient(135deg, #141517 0%, #1a1b1e 100%);",

            Paper {
                shadow: "md",
                p: "xl",
                style: "width: 420px;",

                Stack {
                    gap: "md",

                    Title {
                        order: 2,
                        "Create an Account"
                    }

                    Text {
                        color: "dimmed",
                        size: "sm",
                        "Register with your OFSCP provider"
                    }

                    TextInput {
                        label: "Server URL",
                        placeholder: "localhost:8080",
                        value_fn: move || server_url.get().clone(),
                        oninput: move |val: String| server_url.set(val),
                    }

                    TextInput {
                        label: "Handle",
                        placeholder: "your-handle",
                        value_fn: move || handle_input.get().clone(),
                        oninput: move |val: String| handle_input.set(val),
                    }

                    PasswordInput {
                        label: "Password",
                        placeholder: "Choose a password",
                        value_fn: move || password_input.get().clone(),
                        oninput: move |val: String| password_input.set(val),
                    }

                    PasswordInput {
                        label: "Confirm Password",
                        placeholder: "Confirm your password",
                        value_fn: move || confirm_password.get().clone(),
                        oninput: move |val: String| confirm_password.set(val),
                    }

                    if error_msg.get().is_some() {
                        Alert {
                            color: "red",
                            variant: "light",
                            {error_msg.get().clone().unwrap_or_default()}
                        }
                    }

                    Button {
                        variant: "filled",
                        color: "indigo",
                        full_width: true,
                        disabled: loading.get(),
                        onclick: move || on_register(),
                        {|| if loading.get() { "Creating account...".to_string() } else { "Register".to_string() }}
                    }

                    Group {
                        justify: "center",

                        Text {
                            size: "sm",
                            color: "dimmed",
                            "Already have an account? "
                        }

                        Button {
                            variant: "subtle",
                            onclick: move || navigate(AppRoute::Login),
                            "Sign In"
                        }
                    }
                }
            }
        }
    }
}
