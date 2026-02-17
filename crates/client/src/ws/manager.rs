use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rorumall_shared::{BaseMessage, Presence, ServerEvent, UserRef, WsEnvelope};

use super::connection::{ConnectionState, WsConnection, WsHandle};
use crate::client_keys::sign_ws_request;
use crate::stores::{get_messages_store, get_presence_store, StoredMessage};

pub fn normalize_host(host: &str) -> String {
    host.trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/')
        .to_string()
}

fn extract_user_id(user_ref: &UserRef) -> String {
    match user_ref {
        UserRef::Handle(h) => h.to_string(),
        UserRef::Uri(u) => {
            if let Some(rest) = u.strip_prefix("ofscp://") {
                if let Some(idx) = rest.find("/users/") {
                    let domain = &rest[..idx];
                    let handle = &rest[idx + 7..];
                    return format!("{}@{}", handle, domain);
                }
            }
            u.to_string()
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum WsEvent {
    NewMessage {
        host: String,
        channel_id: String,
        message: BaseMessage,
    },
    PresenceUpdate {
        host: String,
        user_handle: String,
        user_domain: String,
        presence: Presence,
    },
    Ack {
        host: String,
        nonce: String,
        message_id: String,
    },
    Error {
        host: String,
        code: String,
        message: String,
        correlation_id: Option<String>,
    },
    ConnectionStateChanged {
        host: String,
        state: ConnectionState,
    },
}

// Thread-safe global state for WS connections
struct WsManagerState {
    handles: HashMap<String, WsHandle>,
    states: HashMap<String, ConnectionState>,
    connections: HashMap<String, Arc<WsConnection>>,
    requested_hosts: Vec<String>,
}

impl WsManagerState {
    fn new() -> Self {
        Self {
            handles: HashMap::new(),
            states: HashMap::new(),
            connections: HashMap::new(),
            requested_hosts: Vec::new(),
        }
    }
}

static WS_STATE: std::sync::LazyLock<Mutex<WsManagerState>> =
    std::sync::LazyLock::new(|| Mutex::new(WsManagerState::new()));

pub fn request_connection(host: &str) {
    let normalized = normalize_host(host);
    let mut state = WS_STATE.lock().unwrap();
    if !state.requested_hosts.contains(&normalized) {
        state.requested_hosts.push(normalized);
    }
}

pub fn clear_connections() {
    let mut state = WS_STATE.lock().unwrap();
    state.handles.clear();
    state.states.clear();
    state.connections.clear();
    state.requested_hosts.clear();
}

pub fn get_handle(host: &str) -> Option<WsHandle> {
    let normalized = normalize_host(host);
    let state = WS_STATE.lock().unwrap();
    state.handles.get(&normalized).cloned()
}

pub fn get_state(host: &str) -> ConnectionState {
    let normalized = normalize_host(host);
    let state = WS_STATE.lock().unwrap();
    state
        .states
        .get(&normalized)
        .cloned()
        .unwrap_or(ConnectionState::Disconnected)
}

pub fn is_connected(host: &str) -> bool {
    get_state(host).is_connected()
}

pub fn connect_to_host(
    host: &str,
    user_id: &str,
    handle: &str,
    domain: &str,
    keys: &crate::client_keys::KeyPair,
) {
    let normalized = normalize_host(host);

    {
        let state = WS_STATE.lock().unwrap();
        if state.connections.contains_key(&normalized) {
            return;
        }
    }

    let ws_path = "/api/ws";
    let host_option = if normalized.is_empty() {
        None
    } else {
        Some(normalized.as_str())
    };
    let ws_base_url = crate::auth_session::ws_url_for_host(domain, host_option, ws_path);

    let keys_clone = keys.clone();
    let handle_str = handle.to_string();
    let domain_str = crate::auth_session::normalize_domain(domain);

    let url_builder = move || {
        let auth_params = sign_ws_request(ws_path, &keys_clone, &handle_str, &domain_str)?;
        Some(format!("{}?{}", ws_base_url, auth_params.to_query_string()))
    };

    let host_for_event = normalized.clone();
    let user_id_for_event = user_id.to_string();

    let on_event = move |envelope: WsEnvelope<ServerEvent>| {
        match envelope.payload {
            ServerEvent::MessageNew {
                channel_id,
                message,
            } => {
                let stored = StoredMessage {
                    id: message.id.clone(),
                    user_id: extract_user_id(&message.author),
                    title: message.title.clone(),
                    content: message.content.text.clone(),
                    message_type: message.r#type.clone(),
                    created_at: message.created_at,
                    parent_id: message.parent_id.clone(),
                    parent_message_type: message.parent_message_type.clone(),
                    attachments: message.attachments.clone(),
                };

                let _is_own_message = stored.user_id == user_id_for_event;

                rinch::run_on_main_thread(move || {
                    get_messages_store().add_message(&channel_id, stored);
                });
            }
            ServerEvent::PresenceUpdate {
                user_handle,
                user_domain,
                presence,
            } => {
                rinch::run_on_main_thread(move || {
                    get_presence_store().update_user(&user_handle, &user_domain, presence);
                });
            }
            ServerEvent::Ack { nonce, message_id } => {
                tracing::debug!(
                    "WS Ack from {}: nonce={}, message_id={}",
                    host_for_event,
                    nonce,
                    message_id
                );
            }
            ServerEvent::Error {
                code,
                message,
                correlation_id,
            } => {
                tracing::error!(
                    "WS Error from {}: code={}, message={}, correlation_id={:?}",
                    host_for_event,
                    code,
                    message,
                    correlation_id
                );
            }
        }
    };

    let connection = WsConnection::new(normalized.clone(), url_builder, on_event);
    let ws_handle = connection.handle();

    let mut state = WS_STATE.lock().unwrap();
    state
        .handles
        .insert(normalized.clone(), ws_handle);
    state
        .connections
        .insert(normalized.clone(), Arc::new(connection));
    state
        .states
        .insert(normalized, ConnectionState::Connecting);
}
