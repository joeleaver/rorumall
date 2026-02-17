use rinch_core::element::ThemeProviderProps;

pub fn app_theme() -> ThemeProviderProps {
    ThemeProviderProps {
        primary_color: Some("indigo".into()),
        default_radius: Some("md".into()),
        dark_mode: true,
        ..Default::default()
    }
}

pub const CUSTOM_CSS: &str = r#"
body {
    margin: 0;
    padding: 0;
    overflow: hidden;
}

.app-root {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--rinch-color-body);
    color: var(--rinch-color-text);
}

.sidebar {
    background: var(--rinch-color-dark-7, #1a1b1e);
    border-right: 1px solid var(--rinch-color-dark-4, #373a40);
    overflow-y: auto;
}

.channel-sidebar {
    background: var(--rinch-color-dark-6, #25262b);
    border-right: 1px solid var(--rinch-color-dark-4, #373a40);
    overflow-y: auto;
}

.content-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.message-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--rinch-spacing-md);
}

.message-input-area {
    border-top: 1px solid var(--rinch-color-dark-4, #373a40);
    padding: var(--rinch-spacing-sm) var(--rinch-spacing-md);
}

.auth-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: linear-gradient(135deg, var(--rinch-color-dark-8, #141517) 0%, var(--rinch-color-dark-7, #1a1b1e) 100%);
}

.auth-card {
    width: 400px;
    padding: var(--rinch-spacing-xl);
}

.presence-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: inline-block;
}

.presence-online { background: #40c057; }
.presence-away { background: #fab005; }
.presence-dnd { background: #fa5252; }
.presence-offline { background: #868e96; }

.markdown-content h1, .markdown-content h2, .markdown-content h3 {
    margin-top: var(--rinch-spacing-sm);
    margin-bottom: var(--rinch-spacing-xs);
}

.markdown-content p {
    margin: var(--rinch-spacing-xs) 0;
}

.markdown-content code {
    background: var(--rinch-color-dark-5, #2c2e33);
    padding: 2px 6px;
    border-radius: var(--rinch-radius-sm);
    font-size: var(--rinch-font-size-sm);
}

.markdown-content pre {
    background: var(--rinch-color-dark-5, #2c2e33);
    padding: var(--rinch-spacing-sm);
    border-radius: var(--rinch-radius-md);
    overflow-x: auto;
}

.markdown-content blockquote {
    border-left: 3px solid var(--rinch-color-indigo-6);
    padding-left: var(--rinch-spacing-sm);
    margin-left: 0;
    color: var(--rinch-color-dimmed);
}
"#;
