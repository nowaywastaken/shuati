use std::path::PathBuf;
use tauri::api::path::app_local_data_dir;

#[cfg(target_os = "macos")]
const PLATFORM: &str = "macos";

#[cfg(target_os = "windows")]
const PLATFORM: &str = "windows";

#[cfg(target_os = "linux")]
const PLATFORM: &str = "linux";

pub fn get_app_data_dir() -> PathBuf {
    app_local_data_dir(&tauri::Config {
        product_name: Some("刷题神器".to_string()),
        ..Default::default()
    }).unwrap_or_else(|| {
        let mut path = PathBuf::from(env!("HOME"));
        path.push(".local/share/shuati");
        path
    })
}

pub fn get_database_path() -> PathBuf {
    let mut path = get_app_data_dir();
    path.push("shuati.db");
    path
}

pub fn get_models_dir() -> PathBuf {
    let mut path = get_app_data_dir();
    path.push("models");
    path
}

pub fn get_platform() -> &'static str {
    PLATFORM
}

#[cfg(target_arch = "aarch64")]
pub fn get_arch() -> &'static str {
    "arm64"
}

#[cfg(target_arch = "x86_64")]
pub fn get_arch() -> &'static str {
    "x86_64"
}
