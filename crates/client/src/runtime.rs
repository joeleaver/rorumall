//! Async runtime bridge for spawning background work from UI handlers.
//!
//! Rinch signals use thread-local storage, so they must only be accessed from
//! the main (UI) thread.  This module provides [`spawn`] which runs an async
//! future on a tokio worker thread and then executes a callback on the main
//! thread via [`rinch::run_on_main_thread`].

use std::sync::OnceLock;

static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Initialise the global tokio runtime.  Call once from `main()`.
pub fn init() {
    RT.set(
        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"),
    )
    .ok();
}

/// Spawn an async future on the tokio runtime.
///
/// When the future completes, `on_done` runs **on the main (UI) thread** so
/// it is safe to call `Signal::set` / `Signal::update` inside it.
///
/// # Example
///
/// ```ignore
/// crate::runtime::spawn(
///     async move {
///         client.post_json::<_, Resp>(&url, &body).await
///     },
///     move |result| {
///         match result {
///             Ok(resp) => { /* update signals */ },
///             Err(e)   => { error_msg.set(Some(e.to_string())); },
///         }
///         loading.set(false);
///     },
/// );
/// ```
pub fn spawn<F, T, C>(future: F, on_done: C)
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
    C: FnOnce(T) + Send + 'static,
{
    let rt = RT.get().expect("runtime::init() not called");
    rt.spawn(async move {
        let result = future.await;
        rinch::run_on_main_thread(move || {
            on_done(result);
        });
    });
}
