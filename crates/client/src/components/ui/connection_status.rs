use rinch::prelude::*;
use rinch_tabler_icons::{render_tabler_icon, TablerIcon, TablerIconStyle};

#[component]
pub fn connection_status(connected: bool, host: String) -> NodeHandle {
    let (color, icon, label) = if connected {
        (
            "green".to_string(),
            TablerIcon::Wifi,
            format!("Connected to {}", host),
        )
    } else {
        (
            "red".to_string(),
            TablerIcon::WifiOff,
            format!("Disconnected from {}", host),
        )
    };

    rsx! {
        Tooltip {
            label: {label},

            Badge {
                variant: "dot",
                color: {color},
                size: "xs",
                {render_tabler_icon(__scope, icon, TablerIconStyle::Outline)}
            }
        }
    }
}
