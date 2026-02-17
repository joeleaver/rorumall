use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};
use crate::navigation::{get_nav, navigate, AppRoute};
use crate::stores::get_auth_store;
use crate::stores::get_groups_store;

#[component]
pub fn home_view() -> NodeHandle {
    let auth = get_auth_store();
    let groups_store = get_groups_store();
    let loading = use_signal(|| true);
    let show_create_group = use_signal(|| false);
    let show_create_channel = use_signal(|| false);
    let create_channel_gid = use_signal(|| String::new());

    // Check auth
    if !auth.is_authenticated() {
        navigate(AppRoute::Login);
    }

    // Load joined groups on mount
    use_mount(move || {
        let client = auth.make_client();
        let user_id = auth.user_id().unwrap_or_default();

        crate::runtime::spawn(
            async move {
                let path = format!("/api/users/{}/groups", urlencoding::encode(&user_id));
                client.get_json::<Vec<rorumall_shared::UserJoinedGroup>>(&path).await
            },
            move |result| {
                match result {
                    Ok(groups) => {
                        get_groups_store().set_joined_groups(groups);

                        // Connect WS to home provider
                        let domain = get_auth_store().domain();
                        if let Some(session) = get_auth_store().session.get().as_ref() {
                            if let Some(keys) = &session.keys {
                                let handle = session.user_id.split('@').next().unwrap_or(&session.user_id);
                                crate::ws::manager::connect_to_host(
                                    &domain, &session.user_id, handle, &domain, keys,
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to load groups: {}", e);
                    }
                }
                loading.set(false);
            },
        );
        || {}
    });

    let nav = get_nav();

    rsx! {
        div {
            style: "position: relative; width: 100%; height: 100vh;",

            // Main flex layout
            div {
                style: "display: flex; overflow: hidden; height: 100%;",

                // Group sidebar
                div {
                    class: "sidebar",
                    style: "width: 72px; display: flex; flex-direction: column; align-items: center; padding: 8px 0; gap: 8px;",

                    // Home button
                    ActionIcon {
                        variant: "subtle",
                        size: "xl",
                        onclick: move || navigate(AppRoute::Home),
                        {render_tabler_icon(__scope, TablerIcon::Home, TablerIconStyle::Outline)}
                    }

                    Divider {}

                    // Group list
                    for group in groups_store.joined_groups.get().clone() {
                        let _gid = group.group_id.clone();
                        let _host = group.host.clone().unwrap_or_default();

                        div {
                            key: group.group_id.clone(),
                            style: "cursor: pointer;",
                            onclick: move || navigate(AppRoute::Group { host: _host.clone(), group_id: _gid.clone() }),

                            Tooltip {
                                label: group.name.clone(),
                                position: "right",

                                Avatar {
                                    size: "md",
                                    color: "indigo",
                                    src: {group.avatar.clone().unwrap_or_default()},
                                    {group.name.chars().next().unwrap_or('?').to_string()}
                                }
                            }
                        }
                    }

                    // Spacer
                    div { style: "flex: 1;" }

                    // Connection status
                    {crate::components::ui::connection_status::connection_status(
                        __scope,
                        crate::ws::is_connected(&get_auth_store().domain()),
                        get_auth_store().domain(),
                    )}

                    // Add group button
                    ActionIcon {
                        variant: "light",
                        color: "indigo",
                        size: "lg",
                        onclick: move || show_create_group.set(true),
                        {render_tabler_icon(__scope, TablerIcon::Plus, TablerIconStyle::Outline)}
                    }

                    // Profile button
                    ActionIcon {
                        variant: "subtle",
                        size: "lg",
                        onclick: move || navigate(AppRoute::Profile),
                        {render_tabler_icon(__scope, TablerIcon::User, TablerIconStyle::Outline)}
                    }
                }

                // Channel sidebar (when group selected)
                if matches!(nav.get().clone(), AppRoute::Group { .. } | AppRoute::Channel { .. }) {
                    let (host, gid) = match nav.get().clone() {
                        AppRoute::Group { host, group_id } => (host, group_id),
                        AppRoute::Channel { host, group_id, .. } => (host, group_id),
                        _ => (String::new(), String::new()),
                    };

                    div {
                        class: "channel-sidebar",
                        style: "width: 200px; display: flex; flex-direction: column;",
                        {crate::components::ui::channel_list::channel_list(__scope, host.clone(), gid.clone(), show_create_channel, create_channel_gid)}
                    }
                }

                // Content area
                div {
                    class: "content-area",
                    style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",

                    if matches!(nav.get().clone(), AppRoute::Channel { .. }) {
                        div {
                            style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                            {crate::views::channel_view::channel_view(__scope)}
                        }
                    }

                    if matches!(nav.get().clone(), AppRoute::Group { .. }) {
                        let (gs_host, gs_gid) = match nav.get().clone() {
                            AppRoute::Group { host, group_id } => (host, group_id),
                            _ => (String::new(), String::new()),
                        };

                        div {
                            style: "flex: 1; overflow: auto; padding: 24px;",
                            {crate::components::ui::group_settings::group_settings(__scope, gs_host, gs_gid)}
                        }
                    }

                    if !matches!(nav.get().clone(), AppRoute::Channel { .. } | AppRoute::Group { .. }) {
                        div {
                            style: "flex: 1; display: flex; align-items: center; justify-content: center;",
                            Stack {
                                align: "center",
                                gap: "md",

                                {render_tabler_icon(__scope, TablerIcon::Messages, TablerIconStyle::Outline)}

                                Title {
                                    order: 3,
                                    "Welcome to Rorumall"
                                }

                                Text {
                                    color: "dimmed",
                                    "Select a group from the sidebar to get started"
                                }
                            }
                        }
                    }
                }
            }

            // Modals
            if show_create_group.get() {
                {crate::components::ui::create_group_modal::create_group_modal(__scope, show_create_group)}
            }

            if show_create_channel.get() {
                {crate::components::ui::create_channel_modal::create_channel_modal(
                    __scope,
                    show_create_channel,
                    create_channel_gid.get().clone(),
                )}
            }
        }
    }
}
