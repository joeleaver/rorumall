fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Rorumall...");

    // Initialise the global tokio runtime for background async work.
    rorumall::runtime::init();

    rinch::run_with_theme(
        "Rorumall",
        1280,
        800,
        rorumall::app::app,
        rorumall::theme::app_theme(),
    );
}
