use std::sync::Arc;

use rinch::tray::{TrayIconBuilder, TrayMenu, TrayMenuItem};
use rinch::windows::{close_current_window, hide_current_window, show_current_window};
use rinch_core::element::WindowProps;

mod single_instance;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Single-instance: if another instance is running, tell it to show its window and exit.
    if single_instance::signal_existing_instance() {
        std::process::exit(0);
    }
    single_instance::start_listener();

    tracing::info!("Starting Rorumall...");

    // Initialise the global tokio runtime for background async work.
    rorumall::runtime::init();

    // Build system tray icon.
    let menu = TrayMenu::new()
        .add_item(TrayMenuItem::new("Show Rorumall").on_click(show_current_window))
        .add_separator()
        .add_item(TrayMenuItem::new("Quit").on_click(close_current_window));

    let _tray = TrayIconBuilder::new()
        .with_tooltip("Rorumall")
        .with_icon_png(include_bytes!("../../../rorumall-logo.png"))
        .expect("Failed to load tray icon")
        .with_menu(menu)
        .build()
        .expect("Failed to create system tray icon");

    // Configure window: close button hides to tray instead of quitting.
    let props = WindowProps {
        title: "Rorumall".into(),
        width: 1280,
        height: 800,
        icon: Some(include_bytes!("../../../rorumall-logo.png")),
        on_close_requested: Some(Arc::new(|| {
            hide_current_window();
            false // don't exit
        })),
        ..Default::default()
    };

    rinch::run_with_window_props(
        rorumall::app::app,
        props,
        Some(rorumall::theme::app_theme()),
    );
}
