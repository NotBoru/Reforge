use std::fs;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::{RegKey, RegValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupSource {
    HkcuRun,
    HklmRun,
    HklmRun32,
    UserFolder,
    CommonFolder,
}

impl StartupSource {
    pub fn label(self) -> &'static str {
        match self {
            StartupSource::HkcuRun => "HKCU Run",
            StartupSource::HklmRun => "HKLM Run",
            StartupSource::HklmRun32 => "HKLM Run (x86)",
            StartupSource::UserFolder => "User Startup Folder",
            StartupSource::CommonFolder => "Common Startup Folder",
        }
    }

    pub fn needs_admin(self) -> bool {
        matches!(
            self,
            StartupSource::HklmRun | StartupSource::HklmRun32 | StartupSource::CommonFolder
        )
    }
}

#[derive(Debug, Clone)]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub command: String,
    pub source: StartupSource,
    pub enabled: bool,
    pub file_path: Option<PathBuf>,
}

fn is_approved_enabled(root: winreg::HKEY, approved_subkey: &str, value_name: &str) -> bool {
    let hive = RegKey::predef(root);
    let Ok(key) = hive.open_subkey_with_flags(approved_subkey, KEY_READ) else {
        return true;
    };
    let Ok(val) = key.get_raw_value(value_name) else {
        return true;
    };
    // Windows StartupApproved binary format: first byte 0x02 or 0x06 = Enabled; 0x03 or odd = Disabled
    val.bytes.first().map(|&b| (b & 1) == 0).unwrap_or(true)
}

fn set_approved_state(
    root: winreg::HKEY,
    approved_subkey: &str,
    value_name: &str,
    enable: bool,
) -> Result<(), String> {
    let hive = RegKey::predef(root);
    let (key, _) = hive
        .create_subkey(approved_subkey)
        .map_err(|e| format!("Registry write denied (run as Admin for HKLM items): {e}"))?;

    let mut bytes = vec![0u8; 12];
    bytes[0] = if enable { 0x02 } else { 0x03 };
    let val = RegValue {
        bytes: bytes.into(),
        vtype: REG_BINARY,
    };
    key.set_raw_value(value_name, &val)
        .map_err(|e| format!("Failed to update StartupApproved: {e}"))
}

/// Reads all Windows startup programs directly from Registry + Startup folders in <10ms.
pub fn scan_startup_items() -> Vec<StartupItem> {
    let mut items = Vec::new();

    let reg_sources = [
        (
            HKEY_CURRENT_USER,
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run",
            StartupSource::HkcuRun,
        ),
        (
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run",
            StartupSource::HklmRun,
        ),
        (
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run",
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run32",
            StartupSource::HklmRun32,
        ),
    ];

    for (root, run_path, approved_path, source) in reg_sources {
        let hive = RegKey::predef(root);
        let Ok(key) = hive.open_subkey_with_flags(run_path, KEY_READ) else {
            continue;
        };
        for (name, val) in key.enum_values().flatten() {
            let cmd = val.to_string().trim().to_string();
            if name.trim().is_empty() {
                continue;
            }
            let enabled = is_approved_enabled(root, approved_path, &name);
            items.push(StartupItem {
                id: format!("{}::{}", source.label(), name),
                name,
                command: cmd,
                source,
                enabled,
                file_path: None,
            });
        }
    }

    // User & Common Startup Folders
    let mut folder_sources = Vec::new();
    if let Ok(roaming) = std::env::var("APPDATA") {
        folder_sources.push((
            PathBuf::from(roaming).join(r"Microsoft\Windows\Start Menu\Programs\Startup"),
            HKEY_CURRENT_USER,
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder",
            StartupSource::UserFolder,
        ));
    }
    if let Ok(pd) = std::env::var("PROGRAMDATA") {
        folder_sources.push((
            PathBuf::from(pd).join(r"Microsoft\Windows\Start Menu\Programs\Startup"),
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder",
            StartupSource::CommonFolder,
        ));
    }

    for (dir, root, approved_path, source) in folder_sources {
        if let Ok(entries) = fs::read_dir(&dir) {
            for e in entries.flatten() {
                let p = e.path();
                let fname = e.file_name().to_string_lossy().to_string();
                if fname.eq_ignore_ascii_case("desktop.ini") || !p.is_file() {
                    continue;
                }
                let display = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&fname)
                    .to_string();
                let enabled = is_approved_enabled(root, approved_path, &fname);
                items.push(StartupItem {
                    id: format!("{}::{}", source.label(), fname),
                    name: display,
                    command: p.to_string_lossy().to_string(),
                    source,
                    enabled,
                    file_path: Some(p),
                });
            }
        }
    }

    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    items
}

/// Enables or disables a startup item using Windows Task Manager's native StartupApproved flag.
pub fn toggle_startup_item(item: &StartupItem, enable: bool) -> Result<(), String> {
    match item.source {
        StartupSource::HkcuRun => set_approved_state(
            HKEY_CURRENT_USER,
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run",
            &item.name,
            enable,
        ),
        StartupSource::HklmRun => set_approved_state(
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run",
            &item.name,
            enable,
        ),
        StartupSource::HklmRun32 => set_approved_state(
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run32",
            &item.name,
            enable,
        ),
        StartupSource::UserFolder => {
            let fname = item
                .file_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or(&item.name);
            set_approved_state(
                HKEY_CURRENT_USER,
                "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder",
                fname,
                enable,
            )
        }
        StartupSource::CommonFolder => {
            let fname = item
                .file_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or(&item.name);
            set_approved_state(
                HKEY_LOCAL_MACHINE,
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder",
                fname,
                enable,
            )
        }
    }
}

/// Permanently removes a startup entry from Registry or Startup folder.
pub fn delete_startup_item(item: &StartupItem) -> Result<(), String> {
    match item.source {
        StartupSource::HkcuRun => {
            let k = RegKey::predef(HKEY_CURRENT_USER)
                .open_subkey_with_flags(
                    "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    KEY_SET_VALUE,
                )
                .map_err(|e| e.to_string())?;
            k.delete_value(&item.name).map_err(|e| e.to_string())
        }
        StartupSource::HklmRun => {
            let k = RegKey::predef(HKEY_LOCAL_MACHINE)
                .open_subkey_with_flags(
                    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                    KEY_SET_VALUE,
                )
                .map_err(|e| format!("Admin privileges required: {e}"))?;
            k.delete_value(&item.name).map_err(|e| e.to_string())
        }
        StartupSource::HklmRun32 => {
            let k = RegKey::predef(HKEY_LOCAL_MACHINE)
                .open_subkey_with_flags(
                    "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run",
                    KEY_SET_VALUE,
                )
                .map_err(|e| format!("Admin privileges required: {e}"))?;
            k.delete_value(&item.name).map_err(|e| e.to_string())
        }
        StartupSource::UserFolder | StartupSource::CommonFolder => {
            if let Some(p) = &item.file_path {
                fs::remove_file(p).map_err(|e| e.to_string())
            } else {
                Err("Missing startup file path".into())
            }
        }
    }
}
