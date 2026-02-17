use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};
use crate::stores::get_auth_store;

#[component]
pub fn avatar_uploader(
    current_url: String,
    avatar_result: Signal<Option<String>>,
) -> NodeHandle {
    let uploading = use_signal(|| false);

    let on_pick = move || {
        uploading.set(true);
        let client = get_auth_store().make_client();
        crate::runtime::spawn(
            async move {
                if let Some(path) = rinch::dialogs::open_file()
                    .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                    .pick_file()
                {
                    match std::fs::read(&path) {
                        Ok(data) => {
                            let filename = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "avatar.png".to_string());
                            let mime = if filename.ends_with(".png") {
                                "image/png"
                            } else {
                                "image/jpeg"
                            };
                            match client.upload_avatar(data, &filename, mime).await {
                                Ok(response) => return (Some(response.url), None::<String>),
                                Err(e) => return (None, Some(format!("Avatar upload failed: {}", e))),
                            }
                        }
                        Err(e) => return (None, Some(format!("Failed to read file: {}", e))),
                    }
                }
                (None, None)
            },
            move |(new_url, error): (Option<String>, Option<String>)| {
                if let Some(e) = error {
                    tracing::error!("{}", e);
                }
                if let Some(url) = new_url {
                    avatar_result.set(Some(url));
                }
                uploading.set(false);
            },
        );
    };

    rsx! {
        Stack {
            gap: "sm",
            style: "align-items: center;",

            Avatar {
                size: "xl",
                radius: "xl",
                color: "indigo",
                src: {current_url},
                {render_tabler_icon(__scope, TablerIcon::User, TablerIconStyle::Outline)}
            }

            Button {
                variant: "subtle",
                size: "xs",
                loading: uploading.get(),
                onclick: move || on_pick(),
                "Change Avatar"
            }
        }
    }
}
