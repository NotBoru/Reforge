use crate::i18n::Lang;
use crate::ui::theme::ThemeMode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableConfig {
    pub theme: ThemeMode,
    pub lang: Lang,
    pub max_concurrent_installs: usize,
    pub auto_cleanup_downloads: bool,
    pub prefer_quiet_uninstall: bool,
}

impl Default for PortableConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            lang: Lang::En,
            max_concurrent_installs: 4,
            auto_cleanup_downloads: true,
            prefer_quiet_uninstall: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReforgeProfile {
    pub version: String,
    pub selected_app_ids: Vec<String>,
    pub enabled_tweaks: Vec<String>,
}

pub fn portable_config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("reforge.json")))
        .unwrap_or_else(|| PathBuf::from("reforge.json"))
}

pub fn portable_profile_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("reforge-profile.json")))
        .unwrap_or_else(|| PathBuf::from("reforge-profile.json"))
}

pub fn load_config() -> PortableConfig {
    let path = portable_config_path();
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(cfg: &PortableConfig) {
    let path = portable_config_path();
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(path, json);
    }
}

pub fn export_profile(profile: &ReforgeProfile) -> Result<PathBuf, String> {
    let path = portable_profile_path();
    let json = serde_json::to_string_pretty(profile).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn import_profile() -> Result<ReforgeProfile, String> {
    let path = portable_profile_path();
    let raw = fs::read_to_string(&path).map_err(|_| {
        format!(
            "No profile found at {} — export a profile first.",
            path.display()
        )
    })?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid profile JSON: {e}"))
}
