use rinch::prelude::*;
use crate::stores::StoredMessage;

pub fn render_markdown(text: &str) -> String {
    let parser = pulldown_cmark::Parser::new(text);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    ammonia::clean(&html_output)
}

#[component]
pub fn article_item(msg: StoredMessage) -> NodeHandle {
    let user_display = msg.user_id.split('@').next().unwrap_or(&msg.user_id).to_string();
    let time = msg.created_at.format("%b %d, %Y at %H:%M").to_string();
    let expanded = use_signal(|| false);

    let title = msg.title.clone().unwrap_or_else(|| "Untitled Article".to_string());
    let content = msg.content.clone();
    let attachments = use_signal(|| msg.attachments.clone());
    let preview = if content.len() > 200 {
        format!("{}...", &content[..200])
    } else {
        content.clone()
    };

    rsx! {
        Card {
            shadow: "sm",
            p: "lg",
            style: "margin: 12px 0; border-left: 3px solid var(--rinch-color-indigo-6);",

            Stack {
                gap: "sm",

                Group {
                    justify: "space-between",

                    Badge {
                        variant: "light",
                        color: "indigo",
                        "Article"
                    }

                    Text {
                        size: "xs",
                        color: "dimmed",
                        {time}
                    }
                }

                Title {
                    order: 4,
                    {title}
                }

                Group {
                    gap: "xs",

                    Avatar {
                        size: "xs",
                        color: "indigo",
                        {user_display.chars().next().unwrap_or('?').to_string()}
                    }

                    Text {
                        size: "sm",
                        color: "dimmed",
                        {user_display}
                    }
                }

                if expanded.get() {
                    div {
                        class: "markdown-content",
                        {render_markdown(&content)}
                    }
                } else {
                    Text {
                        size: "sm",
                        color: "dimmed",
                        {preview.clone()}
                    }
                }

                Button {
                    variant: "subtle",
                    size: "xs",
                    onclick: move || expanded.update(|v| *v = !*v),
                    {|| if expanded.get() { "Show less".to_string() } else { "Read more".to_string() }}
                }

                // Attachments
                if !attachments.get().is_empty() {
                    Divider {}
                    div {
                        style: "display: flex; flex-wrap: wrap; gap: 8px;",
                        for att in attachments.get().clone() {
                            {crate::components::ui::attachment_display::attachment_display(__scope, att)}
                        }
                    }
                }
            }
        }
    }
}
