use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};
use crate::navigation::{navigate, AppRoute};
use crate::stores::{get_auth_store, get_groups_store, get_members_store};

#[component]
pub fn channel_list(host: String, group_id: String, show_create_channel: Signal<bool>, create_channel_gid: Signal<String>) -> NodeHandle {
    let groups_store = get_groups_store();

    // Get group name
    let group_name = groups_store
        .joined_groups
        .get()
        .iter()
        .find(|g| g.group_id == group_id)
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "Group".to_string());

    // Fetch channels, members, and roles from API on mount
    let gid = group_id.clone();
    use_mount(move || {
        let client = get_auth_store().make_client();
        let gid = gid.clone();

        crate::runtime::spawn(
            async move {
                let channels = client.get_channels(&gid).await;
                let members = client.list_group_members(&gid).await;
                let roles = client.list_roles(&gid).await;
                (gid, channels, members, roles)
            },
            move |(gid, channels, members, roles)| {
                if let Ok(channels) = channels {
                    tracing::info!("Loaded {} channels", channels.len());
                    get_groups_store().set_channels(channels);
                }
                if let Ok(members_resp) = members {
                    get_members_store().set_group_members(&gid, members_resp.members);
                }
                if let Ok(roles_resp) = roles {
                    get_members_store().set_group_roles(&gid, roles_resp.roles);
                }
            },
        );
        || {}
    });

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%;",

            // Group header
            div {
                style: "padding: 12px; display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--rinch-color-dark-4, #373a40);",

                Text {
                    size: "sm",
                    weight: "600",
                    {group_name}
                }

                ActionIcon {
                    variant: "subtle",
                    size: "sm",
                    onclick: {
                        let gid = group_id.clone();
                        move || {
                            create_channel_gid.set(gid.clone());
                            show_create_channel.set(true);
                        }
                    },
                    {render_tabler_icon(__scope, TablerIcon::Plus, TablerIconStyle::Outline)}
                }
            }

            // Channel list
            Stack {
                gap: "0",
                style: "flex: 1; overflow-y: auto;",

                for ch in groups_store.channels.get().clone() {
                    NavLink {
                        key: ch.id.clone(),
                        label: {ch.name.clone()},
                        left_section: Some(TablerIcon::Hash),
                        onclick: {
                            let h = host.clone();
                            let gid = group_id.clone();
                            let cid = ch.id.clone();
                            move || {
                                navigate(AppRoute::Channel {
                                    host: h.clone(),
                                    group_id: gid.clone(),
                                    channel_id: cid.clone(),
                                });
                            }
                        },
                    }
                }
            }

        }
    }
}
