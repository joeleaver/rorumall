use chrono::Utc;
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_util::{SinkExt, StreamExt};
use rorumall_shared::{ClientCommand, MessageType, ServerEvent, WsEnvelope};
use std::sync::{Arc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting { attempt: u32 },
    Failed { reason: String },
}

impl ConnectionState {
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionState::Connected)
    }

    pub fn is_connecting(&self) -> bool {
        matches!(
            self,
            ConnectionState::Connecting | ConnectionState::Reconnecting { .. }
        )
    }
}

#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u32,
    pub max_delay_ms: u32,
    pub backoff_multiplier: f32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: 10,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 1.5,
        }
    }
}

impl ReconnectConfig {
    pub fn delay_for_attempt(&self, attempt: u32) -> u32 {
        let delay = self.initial_delay_ms as f32 * self.backoff_multiplier.powi(attempt as i32);
        (delay as u32).min(self.max_delay_ms)
    }
}

#[derive(Clone)]
pub struct SharedState<T>(Arc<RwLock<T>>);

impl<T: Clone> SharedState<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    pub fn set(&self, value: T) {
        if let Ok(mut guard) = self.0.write() {
            *guard = value;
        }
    }

    pub fn get(&self) -> T {
        self.0
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| panic!("SharedState poisoned"))
    }
}

#[derive(Clone)]
pub struct WsHandle {
    sender: UnboundedSender<WsEnvelope<ClientCommand>>,
    pub host: String,
}

impl WsHandle {
    pub(crate) fn new(sender: UnboundedSender<WsEnvelope<ClientCommand>>, host: String) -> Self {
        Self { sender, host }
    }

    pub fn send(&self, cmd: ClientCommand) -> Result<(), String> {
        let envelope = WsEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            payload: cmd,
            ts: Utc::now(),
            correlation_id: None,
        };
        self.sender
            .unbounded_send(envelope)
            .map_err(|e| format!("Failed to send: {}", e))
    }

    pub fn send_with_correlation(
        &self,
        cmd: ClientCommand,
        correlation_id: String,
    ) -> Result<(), String> {
        let envelope = WsEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            payload: cmd,
            ts: Utc::now(),
            correlation_id: Some(correlation_id),
        };
        self.sender
            .unbounded_send(envelope)
            .map_err(|e| format!("Failed to send: {}", e))
    }

    pub fn subscribe(&self, channel_id: &str) -> Result<(), String> {
        self.send(ClientCommand::Subscribe {
            channel_id: channel_id.to_string(),
        })
    }

    pub fn unsubscribe(&self, channel_id: &str) -> Result<(), String> {
        self.send(ClientCommand::Unsubscribe {
            channel_id: channel_id.to_string(),
        })
    }

    pub fn send_message(&self, channel_id: &str, body: &str, nonce: &str) -> Result<(), String> {
        self.send(ClientCommand::MessageCreate {
            channel_id: channel_id.to_string(),
            body: body.to_string(),
            nonce: nonce.to_string(),
            title: None,
            message_type: None,
            parent_id: None,
            attachments: vec![],
        })
    }

    pub fn send_message_with_options(
        &self,
        channel_id: &str,
        body: &str,
        nonce: &str,
        title: Option<String>,
        message_type: Option<MessageType>,
        attachments: Vec<rorumall_shared::Attachment>,
    ) -> Result<(), String> {
        self.send(ClientCommand::MessageCreate {
            channel_id: channel_id.to_string(),
            body: body.to_string(),
            nonce: nonce.to_string(),
            title,
            message_type,
            parent_id: None,
            attachments,
        })
    }

    pub fn send_reply(
        &self,
        channel_id: &str,
        body: &str,
        nonce: &str,
        parent_id: &str,
        message_type: Option<MessageType>,
        attachments: Vec<rorumall_shared::Attachment>,
    ) -> Result<(), String> {
        self.send(ClientCommand::MessageCreate {
            channel_id: channel_id.to_string(),
            body: body.to_string(),
            nonce: nonce.to_string(),
            title: None,
            message_type,
            parent_id: Some(parent_id.to_string()),
            attachments,
        })
    }
}

pub struct WsConnection {
    pub host: String,
    pub state: SharedState<ConnectionState>,
    sender: UnboundedSender<WsEnvelope<ClientCommand>>,
    #[allow(dead_code)]
    reconnect_config: ReconnectConfig,
    #[allow(dead_code)]
    url_builder: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    #[allow(dead_code)]
    on_event: Arc<dyn Fn(WsEnvelope<ServerEvent>) + Send + Sync>,
}

impl WsConnection {
    pub fn new(
        host: String,
        url_builder: impl Fn() -> Option<String> + Send + Sync + 'static,
        on_event: impl Fn(WsEnvelope<ServerEvent>) + Send + Sync + 'static,
    ) -> Self {
        let (sender, receiver) = unbounded();
        let state = SharedState::new(ConnectionState::Disconnected);
        let reconnect_config = ReconnectConfig::default();

        let url_builder = Arc::new(url_builder);
        let on_event = Arc::new(on_event);

        let connection = Self {
            host: host.clone(),
            state: state.clone(),
            sender,
            reconnect_config: reconnect_config.clone(),
            url_builder: url_builder.clone(),
            on_event: on_event.clone(),
        };

        start_connection_loop(host, state, receiver, url_builder, on_event, reconnect_config);

        connection
    }

    pub fn handle(&self) -> WsHandle {
        WsHandle::new(self.sender.clone(), self.host.clone())
    }
}

fn start_connection_loop(
    host: String,
    state: SharedState<ConnectionState>,
    receiver: UnboundedReceiver<WsEnvelope<ClientCommand>>,
    url_builder: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    on_event: Arc<dyn Fn(WsEnvelope<ServerEvent>) + Send + Sync>,
    reconnect_config: ReconnectConfig,
) {
    crate::runtime::spawn(
        async move {
            let receiver = std::sync::Arc::new(tokio::sync::Mutex::new(receiver));
            let mut attempt = 0u32;

            loop {
                let Some(url) = url_builder() else {
                    state.set(ConnectionState::Disconnected);
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    continue;
                };

                if attempt == 0 {
                    state.set(ConnectionState::Connecting);
                } else {
                    state.set(ConnectionState::Reconnecting { attempt });
                }

                match connect_async(&url).await {
                    Ok((ws_stream, _response)) => {
                        state.set(ConnectionState::Connected);
                        attempt = 0;
                        tracing::info!("WebSocket connected to {}", host);

                        let (mut write, mut read) = ws_stream.split();
                        let (close_tx, mut close_rx) =
                            tokio::sync::mpsc::unbounded_channel::<()>();

                        // Read task
                        let host_for_read = host.clone();
                        let on_event_clone = on_event.clone();
                        let close_tx_for_read = close_tx.clone();
                        tokio::spawn(async move {
                            while let Some(msg_result) = read.next().await {
                                match msg_result {
                                    Ok(Message::Text(text)) => {
                                        match serde_json::from_str::<WsEnvelope<ServerEvent>>(
                                            &text,
                                        ) {
                                            Ok(event) => on_event_clone(event),
                                            Err(e) => {
                                                tracing::error!(
                                                    "Failed to parse WS message: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    Ok(Message::Close(_)) => {
                                        tracing::info!(
                                            "WebSocket to {} received close frame",
                                            host_for_read
                                        );
                                        break;
                                    }
                                    Ok(Message::Ping(_)) => {}
                                    Ok(_) => {}
                                    Err(e) => {
                                        tracing::error!("WebSocket read error: {}", e);
                                        break;
                                    }
                                }
                            }
                            let _ = close_tx_for_read.send(());
                        });

                        // Write task
                        let receiver_for_write = receiver.clone();
                        let host_for_write = host.clone();
                        tokio::spawn(async move {
                            loop {
                                let msg = {
                                    let mut rx = receiver_for_write.lock().await;
                                    rx.next().await
                                };

                                match msg {
                                    Some(cmd) => match serde_json::to_string(&cmd) {
                                        Ok(json) => {
                                            tracing::debug!(
                                                "Sending to {}: {}",
                                                host_for_write,
                                                json
                                            );
                                            if let Err(e) =
                                                write.send(Message::Text(json.into())).await
                                            {
                                                tracing::error!("Send failed: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Serialize failed: {}", e);
                                        }
                                    },
                                    None => {
                                        tracing::info!("Sender dropped, stopping write task");
                                        break;
                                    }
                                }
                            }
                            let _ = close_tx.send(());
                        });

                        close_rx.recv().await;
                        tracing::info!("WebSocket to {} closed", host);
                        state.set(ConnectionState::Disconnected);
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error for {}: {}", host, e);

                        if reconnect_config.max_attempts > 0
                            && attempt >= reconnect_config.max_attempts
                        {
                            state.set(ConnectionState::Failed {
                                reason: format!(
                                    "Max reconnect attempts ({}) exceeded",
                                    reconnect_config.max_attempts
                                ),
                            });
                            break;
                        }

                        let delay = reconnect_config.delay_for_attempt(attempt);
                        tracing::info!(
                            "Reconnecting to {} in {}ms (attempt {})",
                            host,
                            delay,
                            attempt + 1
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay as u64))
                            .await;
                        attempt += 1;
                    }
                }
            }
        },
        |_| {},
    );
}
