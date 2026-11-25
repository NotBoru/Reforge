use crate::core::winapi;
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
            theme: ThemeMode::Dark,
            lang: if winapi::detect_os_language_is_pt() {
                Lang::PtBr
            } else {
                Lang::En
            },
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

pub fn appdata_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("Reforge").join("config.json"))
}

pub fn portable_profile_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("reforge-profile.json")))
        .unwrap_or_else(|| PathBuf::from("reforge-profile.json"))
}

pub fn load_config() -> PortableConfig {
    let mut best: Option<(std::time::SystemTime, PortableConfig)> = None;
    for path in [Some(portable_config_path()), appdata_config_path()]
        .into_iter()
        .flatten()
    {
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<PortableConfig>(&raw) {
                let mtime = fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::UNIX_EPOCH);
                if best.as_ref().is_none_or(|(t, _)| mtime >= *t) {
                    best = Some((mtime, cfg));
                }
            }
        }
    }
    best.map(|(_, c)| c).unwrap_or_default()
}

pub fn save_config(cfg: &PortableConfig) {
    let Ok(json) = serde_json::to_string_pretty(cfg) else {
        return;
    };
    let _ = fs::write(portable_config_path(), &json);
    if let Some(app_path) = appdata_config_path() {
        if let Some(parent) = app_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(app_path, &json);
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
    let raw = fs::read_to_string(&path)
        .map_err(|_| format!("Profile not found at {}.", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid profile file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_serialization() {
        let cfg = PortableConfig {
            theme: ThemeMode::GeminiDark,
            lang: Lang::PtBr,
            max_concurrent_installs: 6,
            auto_cleanup_downloads: false,
            prefer_quiet_uninstall: true,
        };
        let json = serde_json::to_string(&cfg).expect("serialize config");
        let parsed: PortableConfig = serde_json::from_str(&json).expect("deserialize config");
        assert_eq!(parsed.theme, ThemeMode::GeminiDark);
        assert_eq!(parsed.lang, Lang::PtBr);
        assert_eq!(parsed.max_concurrent_installs, 6);
        assert!(!parsed.auto_cleanup_downloads);
        assert!(parsed.prefer_quiet_uninstall);
    }
}
