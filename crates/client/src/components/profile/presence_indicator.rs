use rinch::prelude::*;

#[component]
pub fn presence_indicator(status: String, size: String) -> NodeHandle {
    let (color, label) = match status.as_str() {
        "online" => ("var(--rinch-color-green-6, #40c057)", "Online"),
        "away" => ("var(--rinch-color-yellow-6, #fab005)", "Away"),
        "busy" | "dnd" => ("var(--rinch-color-red-6, #fa5252)", "Busy"),
        _ => ("var(--rinch-color-dark-3, #495057)", "Offline"),
    };

    let dim = match size.as_str() {
        "lg" => "14px",
        "sm" => "8px",
        _ => "10px",
    };

    rsx! {
        Tooltip {
            label: label.to_string(),
            div {
                class: "presence-dot",
                style: {format!(
                    "width: {}; height: {}; border-radius: 50%; background: {}; border: 2px solid var(--rinch-color-dark-7, #1a1b1e); display: inline-block;",
                    dim, dim, color
                )},
            }
        }
    }
}
