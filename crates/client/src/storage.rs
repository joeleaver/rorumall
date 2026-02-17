use serde::{de::DeserializeOwned, Serialize};

pub fn save<T: Serialize>(key: &str, value: &T) -> bool {
    match serde_json::to_string(value) {
        Ok(json) => save_raw(key, &json),
        Err(_) => false,
    }
}

pub fn load<T: DeserializeOwned>(key: &str) -> Option<T> {
    let json = load_raw(key)?;
    serde_json::from_str(&json).ok()
}

pub fn remove(key: &str) {
    remove_raw(key);
}

pub fn exists(key: &str) -> bool {
    load_raw(key).is_some()
}

fn get_config_dir() -> Option<std::path::PathBuf> {
    let config_dir = dirs::config_dir()?;
    let app_dir = config_dir.join("rorumall");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).ok()?;
    }
    Some(app_dir)
}

fn get_file_path(key: &str) -> Option<std::path::PathBuf> {
    let config_dir = get_config_dir()?;
    let safe_key = key.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    Some(config_dir.join(format!("{}.json", safe_key)))
}

fn save_raw(key: &str, value: &str) -> bool {
    let Some(path) = get_file_path(key) else {
        return false;
    };
    std::fs::write(path, value).is_ok()
}

fn load_raw(key: &str) -> Option<String> {
    let path = get_file_path(key)?;
    std::fs::read_to_string(path).ok()
}

fn remove_raw(key: &str) {
    if let Some(path) = get_file_path(key) {
        let _ = std::fs::remove_file(path);
    }
}
