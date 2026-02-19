//! Single-instance enforcement via a Unix domain socket.
//!
//! The first instance binds a socket at `$XDG_RUNTIME_DIR/rorumall.sock`
//! (or `/tmp/rorumall-<uid>.sock` as fallback). A background thread listens
//! for connections and calls `show_current_window()` on the main thread when
//! one arrives.
//!
//! When a second instance starts, it tries to connect to that socket. If it
//! succeeds, the existing instance is already running — so the second instance
//! exits after signalling.

use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use rinch::windows::show_current_window;

fn socket_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(dir).join("rorumall.sock")
    } else {
        let user = std::env::var("USER").unwrap_or_else(|_| "default".into());
        PathBuf::from(format!("/tmp/rorumall-{}.sock", user))
    }
}

/// Try to connect to an already-running instance. Returns `true` if one was
/// found (and signalled to show), meaning this process should exit.
pub fn signal_existing_instance() -> bool {
    let path = socket_path();
    if UnixStream::connect(&path).is_ok() {
        // Connection succeeded — another instance is listening.
        // Just connecting is enough; the listener treats any connection as "show".
        true
    } else {
        // No listener — clean up stale socket file if present.
        let _ = std::fs::remove_file(&path);
        false
    }
}

/// Bind the socket and spawn a background thread that listens for incoming
/// connections. Each connection triggers `show_current_window()` on the main
/// thread.
pub fn start_listener() {
    let path = socket_path();
    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("Could not bind single-instance socket: {}", e);
            return;
        }
    };

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(_) => {
                    rinch::run_on_main_thread(|| {
                        show_current_window();
                    });
                }
                Err(e) => {
                    tracing::warn!("Single-instance listener error: {}", e);
                }
            }
        }
    });
}
