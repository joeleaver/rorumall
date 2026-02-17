pub mod connection;
pub mod manager;

pub use connection::{ConnectionState, ReconnectConfig, WsConnection, WsHandle};
pub use manager::{
    clear_connections, get_handle, get_state, is_connected, normalize_host, request_connection,
    WsEvent,
};
