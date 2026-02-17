use serde::{Deserialize, Serialize};

use crate::api_client::ApiClient;
use crate::client_keys::KeyPair;

const STORAGE_KEY: &str = "ofscp_session";
const DOMAIN_KEY: &str = "ofscp_provider_domain";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthSession {
    pub user_id: String,
    pub keys: Option<KeyPair>,
}

pub fn load_session() -> Option<AuthSession> {
    crate::storage::load(STORAGE_KEY)
}

pub fn save_session(session: &AuthSession) {
    crate::storage::save(STORAGE_KEY, session);
}

pub fn clear_session() {
    crate::storage::remove(STORAGE_KEY);
}

pub fn load_domain() -> String {
    crate::storage::load::<String>(DOMAIN_KEY).unwrap_or_else(|| "localhost:8080".to_string())
}

pub fn save_domain(domain: &str) {
    crate::storage::save(DOMAIN_KEY, &domain.to_string());
}

pub fn api_base_url(domain: &str) -> String {
    if domain.trim().is_empty() {
        return String::new();
    }
    if domain.contains("://") {
        return domain.trim_end_matches('/').to_string();
    }
    let host_part = domain.split(':').next().unwrap_or(domain);
    let is_local = host_part == "localhost"
        || host_part == "127.0.0.1"
        || host_part == "0.0.0.0"
        || host_part.starts_with("192.168.")
        || host_part.starts_with("10.");
    if is_local {
        format!("http://{}", domain.trim_end_matches('/'))
    } else {
        format!("https://{}", domain.trim_end_matches('/'))
    }
}

pub fn api_url(domain: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    let base = api_base_url(domain);
    if base.is_empty() {
        if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        }
    } else {
        let base = base.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }
}

pub fn api_url_for_host(current_domain: &str, host: Option<&str>, path: &str) -> String {
    let host = host.unwrap_or("");
    let normalized_host = host
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/');
    let normalized_current = current_domain
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/');

    if host.is_empty() || normalized_host == normalized_current {
        return api_url(current_domain, path);
    }

    let host_part = normalized_host.split(':').next().unwrap_or(normalized_host);
    let is_local = host_part == "localhost"
        || host_part == "127.0.0.1"
        || host_part == "0.0.0.0"
        || host_part.starts_with("192.168.")
        || host_part.starts_with("10.");

    let base = if host.contains("://") {
        host.trim_end_matches('/').to_string()
    } else if is_local {
        format!("http://{}", normalized_host)
    } else {
        format!("https://{}", normalized_host)
    };

    let base = base.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

pub fn ws_url(domain: &str, path: &str) -> String {
    http_to_ws(&api_url(domain, path), domain)
}

pub fn ws_url_for_host(current_domain: &str, host: Option<&str>, path: &str) -> String {
    http_to_ws(
        &api_url_for_host(current_domain, host, path),
        current_domain,
    )
}

fn http_to_ws(url: &str, domain: &str) -> String {
    if url.starts_with("https://") {
        url.replacen("https://", "wss://", 1)
    } else if url.starts_with("http://") {
        url.replacen("http://", "ws://", 1)
    } else {
        let origin = api_base_url(domain);
        let ws_origin = if origin.starts_with("https://") {
            origin.replacen("https://", "wss://", 1)
        } else {
            origin.replacen("http://", "ws://", 1)
        };
        format!(
            "{}{}",
            ws_origin.trim_end_matches('/'),
            if url.starts_with('/') {
                url.to_string()
            } else {
                format!("/{url}")
            }
        )
    }
}

/// Strip scheme from a domain string for OFSCP actor identity.
pub fn normalize_domain(domain: &str) -> String {
    domain
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/')
        .to_string()
}

pub fn make_client(session: Option<&AuthSession>, domain: &str) -> ApiClient {
    let handle = session.map(|s| s.user_id.split('@').next().unwrap_or(&s.user_id).to_string());

    ApiClient::new()
        .with_base_url(api_base_url(domain))
        .with_signing(
            session.and_then(|s| s.keys.clone()),
            handle,
            Some(normalize_domain(domain)),
        )
}
