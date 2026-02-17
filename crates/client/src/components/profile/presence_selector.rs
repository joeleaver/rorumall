use rinch::prelude::*;
use crate::stores::{get_auth_store, get_presence_store};

#[component]
pub fn presence_selector() -> NodeHandle {
    let current = get_presence_store().current.get().clone();
    let current_status = current
        .as_ref()
        .map(|p| match p.availability {
            rorumall_shared::Availability::Online => "online",
            rorumall_shared::Availability::Away => "away",
            rorumall_shared::Availability::Dnd => "busy",
            rorumall_shared::Availability::Offline => "offline",
        })
        .unwrap_or("offline")
        .to_string();

    rsx! {
        Stack {
            gap: "xs",

            Text {
                size: "sm",
                weight: "600",
                "Status"
            }

            Group {
                gap: "xs",

                Button {
                    variant: {if current_status == "online" { "filled".to_string() } else { "subtle".to_string() }},
                    size: "xs",
                    color: "green",
                    onclick: move || {
                        let client = get_auth_store().make_client();
                        crate::runtime::spawn(
                            async move {
                                let req = rorumall_shared::UpdatePresenceRequest {
                                    availability: rorumall_shared::Availability::Online,
                                    status: None,
                                };
                                client.update_presence(&req).await
                            },
                            |result| match result {
                                Ok(presence) => { get_presence_store().current.set(Some(presence)); }
                                Err(e) => { tracing::error!("Failed to update presence: {}", e); }
                            },
                        );
                    },
                    "Online"
                }

                Button {
                    variant: {if current_status == "away" { "filled".to_string() } else { "subtle".to_string() }},
                    size: "xs",
                    color: "yellow",
                    onclick: move || {
                        let client = get_auth_store().make_client();
                        crate::runtime::spawn(
                            async move {
                                let req = rorumall_shared::UpdatePresenceRequest {
                                    availability: rorumall_shared::Availability::Away,
                                    status: None,
                                };
                                client.update_presence(&req).await
                            },
                            |result| match result {
                                Ok(presence) => { get_presence_store().current.set(Some(presence)); }
                                Err(e) => { tracing::error!("Failed to update presence: {}", e); }
                            },
                        );
                    },
                    "Away"
                }

                Button {
                    variant: {if current_status == "busy" { "filled".to_string() } else { "subtle".to_string() }},
                    size: "xs",
                    color: "red",
                    onclick: move || {
                        let client = get_auth_store().make_client();
                        crate::runtime::spawn(
                            async move {
                                let req = rorumall_shared::UpdatePresenceRequest {
                                    availability: rorumall_shared::Availability::Dnd,
                                    status: None,
                                };
                                client.update_presence(&req).await
                            },
                            |result| match result {
                                Ok(presence) => { get_presence_store().current.set(Some(presence)); }
                                Err(e) => { tracing::error!("Failed to update presence: {}", e); }
                            },
                        );
                    },
                    "Busy"
                }

                Button {
                    variant: {if current_status == "offline" { "filled".to_string() } else { "subtle".to_string() }},
                    size: "xs",
                    color: "gray",
                    onclick: move || {
                        let client = get_auth_store().make_client();
                        crate::runtime::spawn(
                            async move {
                                let req = rorumall_shared::UpdatePresenceRequest {
                                    availability: rorumall_shared::Availability::Offline,
                                    status: None,
                                };
                                client.update_presence(&req).await
                            },
                            |result| match result {
                                Ok(presence) => { get_presence_store().current.set(Some(presence)); }
                                Err(e) => { tracing::error!("Failed to update presence: {}", e); }
                            },
                        );
                    },
                    "Offline"
                }
            }
        }
    }
}
