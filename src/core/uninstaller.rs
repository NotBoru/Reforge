use crate::core::winapi::hidden_tokio_cmd;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use winreg::enums::*;
use winreg::{HKEY, RegKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegRoot {
    Hklm,
    Hkcu,
}

impl RegRoot {
    pub fn as_str(self) -> &'static str {
        match self {
            RegRoot::Hklm => "HKLM",
            RegRoot::Hkcu => "HKCU",
        }
    }

    pub fn hkey(self) -> HKEY {
        match self {
            RegRoot::Hklm => HKEY_LOCAL_MACHINE,
            RegRoot::Hkcu => HKEY_CURRENT_USER,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub publisher: String,
    pub version: String,
    pub install_date: String,
    pub estimated_size_bytes: u64,
    pub install_location: String,
    pub uninstall_string: String,
    pub quiet_uninstall_string: String,
    pub reg_root: RegRoot,
    pub reg_subkey: String,
    pub is_msi: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceKind {
    Directory,
    File,
    RegistryKey,
    Shortcut,
    ScheduledTask,
    Service,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceConfidence {
    High,
    Medium,
}

#[derive(Debug, Clone)]
pub struct AppTrace {
    pub id: usize,
    pub kind: TraceKind,
    pub confidence: TraceConfidence,
    pub path: String,
    pub size_bytes: u64,
    pub selected: bool,
    pub reg_root: Option<RegRoot>,
}

#[derive(Debug, Clone, Default)]
pub struct TraceDeleteReport {
    pub deleted_count: usize,
    pub failed_count: usize,
    pub freed_bytes: u64,
}

/// Scans Windows Registry (HKLM 64-bit, HKLM WOW6432Node 32-bit, HKCU) in milliseconds.
pub fn scan_installed_apps(include_system: bool) -> Vec<InstalledApp> {
    let hives: &[(RegRoot, &str)] = &[
        (
            RegRoot::Hklm,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
        (
            RegRoot::Hklm,
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
        (
            RegRoot::Hkcu,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ),
    ];

    let mut apps = Vec::with_capacity(256);
    let mut seen = HashSet::new();

    for &(root, base_path) in hives {
        let hive = RegKey::predef(root.hkey());
        let Ok(parent) = hive.open_subkey_with_flags(base_path, KEY_READ) else {
            continue;
        };

        for sub_name in parent.enum_keys().flatten() {
            let Ok(sub) = parent.open_subkey_with_flags(&sub_name, KEY_READ) else {
                continue;
            };

            let name: String = match sub.get_value("DisplayName") {
                Ok(n) => n,
                Err(_) => continue,
            };
            let name = name.trim().to_string();
            if name.is_empty() {
                continue;
            }

            let sys_comp: u32 = sub.get_value("SystemComponent").unwrap_or(0);
            let parent_key: String = sub.get_value("ParentKeyName").unwrap_or_default();
            let release_type: String = sub.get_value("ReleaseType").unwrap_or_default();
            let is_sys = sys_comp == 1
                || !parent_key.is_empty()
                || release_type.eq_ignore_ascii_case("Update")
                || release_type.eq_ignore_ascii_case("Hotfix")
                || release_type.eq_ignore_ascii_case("Security Update");

            if is_sys && !include_system {
                continue;
            }

            let uninstall_string: String = sub.get_value("UninstallString").unwrap_or_default();
            let quiet_uninstall_string: String =
                sub.get_value("QuietUninstallString").unwrap_or_default();
            if uninstall_string.trim().is_empty() && quiet_uninstall_string.trim().is_empty() {
                continue;
            }

            let version: String = sub.get_value("DisplayVersion").unwrap_or_default();
            let publisher: String = sub.get_value("Publisher").unwrap_or_default();
            let install_location: String = sub.get_value("InstallLocation").unwrap_or_default();
            let raw_date: String = sub.get_value("InstallDate").unwrap_or_default();
            let size_kb: u32 = sub.get_value("EstimatedSize").unwrap_or(0);
            let win_inst: u32 = sub.get_value("WindowsInstaller").unwrap_or(0);

            let dedup_key = format!(
                "{}|{}|{}",
                name.to_lowercase(),
                version.to_lowercase(),
                root.as_str()
            );
            if !seen.insert(dedup_key) {
                continue;
            }

            let reg_subkey = format!("{base_path}\\{sub_name}");
            let is_msi = win_inst == 1
                || uninstall_string.to_lowercase().contains("msiexec");

            apps.push(InstalledApp {
                id: format!("{}\\{}", root.as_str(), reg_subkey),
                name,
                publisher: publisher.trim().to_string(),
                version: version.trim().to_string(),
                install_date: format_reg_date(&raw_date),
                estimated_size_bytes: (size_kb as u64) * 1024,
                install_location: install_location.trim().trim_matches('"').to_string(),
                uninstall_string: uninstall_string.trim().to_string(),
                quiet_uninstall_string: quiet_uninstall_string.trim().to_string(),
                reg_root: root,
                reg_subkey,
                is_msi,
            });
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

fn format_reg_date(raw: &str) -> String {
    let s = raw.trim();
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8])
    } else {
        s.to_string()
    }
}

/// Runs the application's native uninstaller (supports silent/quiet mode when available).
pub async fn run_native_uninstaller(app: &InstalledApp, prefer_quiet: bool) -> Result<(), String> {
    let cmdline = if prefer_quiet && !app.quiet_uninstall_string.is_empty() {
        app.quiet_uninstall_string.clone()
    } else if prefer_quiet && app.is_msi {
        let mut s = app.uninstall_string.replace("/I", "/X").replace("/i", "/X");
        if !s.to_lowercase().contains("/q") {
            s.push_str(" /qn /norestart");
        }
        s
    } else {
        app.uninstall_string.clone()
    };

    if cmdline.trim().is_empty() {
        return Err("No uninstall command registered for this application.".into());
    }

    // Use cmd /C with exact command line so quoted/unquoted paths & args with spaces execute faithfully
    let mut cmd = hidden_tokio_cmd("cmd");
    cmd.raw_arg("/C");
    cmd.raw_arg(&cmdline);
    let status = cmd
        .status()
        .await
        .map_err(|e| format!("Failed to launch uninstaller: {e}"))?;

    let code = status.code().unwrap_or(-1);
    if status.success() || code == 3010 || code == 1605 {
        Ok(())
    } else {
        Err(format!(
            "Uninstaller exited with code {code}. You can still run Deep Trace Scan below."
        ))
    }
}

/// Force removes a broken or orphaned uninstall registry entry.
pub fn force_remove_uninstall_entry(app: &InstalledApp) -> Result<(), String> {
    let hive = RegKey::predef(app.reg_root.hkey());
    hive.delete_subkey_all(&app.reg_subkey)
        .map_err(|e| format!("Could not remove registry key (Admin may be required): {e}"))
}

// ── REVO-STYLE DEEP TRACE SCANNER ────────────────────────────────────────────

/// Strict safety denylist so broad vendor or Windows names are NEVER flagged as leftovers.
const PROTECTED_TOKENS: &[&str] = &[
    "windows",
    "microsoft",
    "system",
    "system32",
    "syswow64",
    "program files",
    "program files (x86)",
    "common files",
    "programdata",
    "appdata",
    "roaming",
    "local",
    "locallow",
    "temp",
    "users",
    "public",
    "default",
    "desktop",
    "documents",
    "downloads",
    "intel",
    "nvidia",
    "amd",
    "google",
    "adobe",
    "apple",
    "oracle",
    "mozilla",
    "valve",
    "steam",
    "epic games",
    "meta",
    "software",
    "packages",
    "programs",
    "start menu",
    "driver",
    "drivers",
    "runtime",
    "redistributable",
    "update",
    "helper",
    "service",
    "client",
    "launcher",
    "installer",
    "setup",
    "app",
    "apps",
    "core",
    "data",
    "cache",
    "config",
    "logs",
    "bin",
    "lib",
    "win64",
    "win32",
    "x64",
    "x86",
];

fn normalize_token(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if c.is_whitespace() || c == '-' || c == '_' || c == '.' {
            if !out.ends_with(' ') {
                out.push(' ');
            }
        }
    }
    out.trim().to_string()
}

fn strip_version_suffix(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    let mut kept = Vec::new();
    for p in parts {
        let lower = p.to_lowercase();
        let is_ver = p.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
            && (p.contains('.') || p.len() <= 5);
        let is_arch = matches!(
            lower.as_str(),
            "(64-bit)" | "(32-bit)" | "64-bit" | "32-bit" | "x64" | "x86" | "(x64)" | "(x86)" | "(user)"
        );
        if is_ver || is_arch {
            break;
        }
        kept.push(p);
    }
    if kept.is_empty() {
        name.to_string()
    } else {
        kept.join(" ")
    }
}

fn is_safe_match_token(token: &str) -> bool {
    let norm = normalize_token(token);
    if norm.len() < 4 {
        return false;
    }
    let compact = norm.replace(' ', "");
    if compact.len() < 4 {
        return false;
    }
    !PROTECTED_TOKENS.iter().any(|&p| p == norm || p == compact)
}

fn build_search_tokens(app: &InstalledApp) -> Vec<String> {
    let mut tokens = Vec::new();
    let base_name = strip_version_suffix(&app.name);
    for candidate in [app.name.as_str(), base_name.as_str()] {
        let norm = normalize_token(candidate);
        if is_safe_match_token(&norm) && !tokens.contains(&norm) {
            tokens.push(norm.clone());
        }
        let compact = norm.replace(' ', "");
        if is_safe_match_token(&compact) && !tokens.contains(&compact) {
            tokens.push(compact);
        }
    }

    if !app.install_location.is_empty() {
        if let Some(folder) = Path::new(&app.install_location)
            .file_name()
            .and_then(|n| n.to_str())
        {
            let norm = normalize_token(folder);
            if is_safe_match_token(&norm) && !tokens.contains(&norm) {
                tokens.push(norm);
            }
        }
    }

    tokens
}

fn match_name(entry_name: &str, tokens: &[String]) -> Option<TraceConfidence> {
    let norm = normalize_token(entry_name);
    if !is_safe_match_token(&norm) {
        return None;
    }
    let compact = norm.replace(' ', "");
    for t in tokens {
        let t_compact = t.replace(' ', "");
        if norm == *t || compact == t_compact {
            return Some(TraceConfidence::High);
        }
        if t_compact.len() >= 5 && (norm.contains(t.as_str()) || compact.contains(&t_compact)) {
            return Some(TraceConfidence::Medium);
        }
    }
    None
}

fn dir_size_bytes(path: &Path) -> u64 {
    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    WalkDir::new(path)
        .max_depth(6)
        .into_iter()
        .flatten()
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

/// Revo-style parallel leftover trace scan across Filesystem, Registry, Shortcuts, Tasks & Services.
pub fn deep_scan_traces(app: &InstalledApp) -> Vec<AppTrace> {
    let tokens = build_search_tokens(app);
    let mut traces: Vec<AppTrace> = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    // 1. Check original InstallLocation directly (High confidence!)
    if !app.install_location.is_empty() {
        let loc = PathBuf::from(&app.install_location);
        if loc.exists() && is_safe_install_dir(&loc) {
            let key = loc.to_string_lossy().to_lowercase();
            if seen_paths.insert(key) {
                traces.push(AppTrace {
                    id: 0,
                    kind: TraceKind::Directory,
                    confidence: TraceConfidence::High,
                    path: loc.to_string_lossy().to_string(),
                    size_bytes: dir_size_bytes(&loc),
                    selected: true,
                    reg_root: None,
                });
            }
        }
    }

    // 2. Parallel filesystem scan across AppData, LocalAppData, Programs, ProgramData, Program Files
    let mut scan_roots: Vec<(PathBuf, usize)> = Vec::new();
    if let Ok(v) = std::env::var("APPDATA") {
        scan_roots.push((PathBuf::from(v), 2));
    }
    if let Ok(v) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(&v);
        scan_roots.push((p.join("Programs"), 2));
        scan_roots.push((p, 2));
    }
    if let Ok(v) = std::env::var("PROGRAMDATA") {
        scan_roots.push((PathBuf::from(v), 2));
    }
    scan_roots.push((PathBuf::from(r"C:\Program Files"), 2));
    scan_roots.push((PathBuf::from(r"C:\Program Files (x86)"), 2));

    let fs_hits: Vec<AppTrace> = scan_roots
        .par_iter()
        .flat_map(|(root, max_depth)| {
            let mut local_hits = Vec::new();
            if !root.exists() {
                return local_hits;
            }
            let Ok(entries) = fs::read_dir(root) else {
                return local_hits;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(conf) = match_name(&name, &tokens) {
                    if is_safe_install_dir(&path) {
                        local_hits.push(AppTrace {
                            id: 0,
                            kind: if path.is_dir() {
                                TraceKind::Directory
                            } else {
                                TraceKind::File
                            },
                            confidence: conf,
                            path: path.to_string_lossy().to_string(),
                            size_bytes: dir_size_bytes(&path),
                            selected: conf == TraceConfidence::High,
                            reg_root: None,
                        });
                        continue;
                    }
                }
                // Check one level deeper for Publisher\AppName folders (e.g. AppData\Local\Google\Chrome)
                if *max_depth >= 2 && path.is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub in sub_entries.flatten() {
                            let spath = sub.path();
                            let sname = sub.file_name().to_string_lossy().to_string();
                            if let Some(conf) = match_name(&sname, &tokens) {
                                if is_safe_install_dir(&spath) {
                                    local_hits.push(AppTrace {
                                        id: 0,
                                        kind: if spath.is_dir() {
                                            TraceKind::Directory
                                        } else {
                                            TraceKind::File
                                        },
                                        confidence: conf,
                                        path: spath.to_string_lossy().to_string(),
                                        size_bytes: dir_size_bytes(&spath),
                                        selected: conf == TraceConfidence::High,
                                        reg_root: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            local_hits
        })
        .collect();

    for hit in fs_hits {
        let lower = hit.path.to_lowercase();
        // Skip if parent folder is already listed
        let covered = seen_paths.iter().any(|p| lower == *p || lower.starts_with(&format!("{p}\\")));
        if !covered && seen_paths.insert(lower) {
            traces.push(hit);
        }
    }

    // 3. Shortcuts (Start Menu & Desktop)
    let mut shortcut_dirs = Vec::new();
    if let Ok(v) = std::env::var("APPDATA") {
        shortcut_dirs.push(PathBuf::from(v).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    if let Ok(v) = std::env::var("PROGRAMDATA") {
        shortcut_dirs.push(PathBuf::from(v).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    if let Some(d) = dirs::desktop_dir() {
        shortcut_dirs.push(d);
    }
    shortcut_dirs.push(PathBuf::from(r"C:\Users\Public\Desktop"));

    for sdir in shortcut_dirs {
        if !sdir.exists() {
            continue;
        }
        for entry in WalkDir::new(&sdir).max_depth(2).into_iter().flatten() {
            let p = entry.path();
            if p == sdir {
                continue;
            }
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if let Some(conf) = match_name(stem, &tokens) {
                let lower = p.to_string_lossy().to_lowercase();
                if seen_paths.insert(lower) {
                    traces.push(AppTrace {
                        id: 0,
                        kind: TraceKind::Shortcut,
                        confidence: conf,
                        path: p.to_string_lossy().to_string(),
                        size_bytes: dir_size_bytes(p),
                        selected: conf == TraceConfidence::High,
                        reg_root: None,
                    });
                }
            }
        }
    }

    // 4. Registry Leftovers (HKCU\Software, HKLM\SOFTWARE, WOW6432Node, orphaned Uninstall key)
    let reg_roots: &[(RegRoot, &str)] = &[
        (RegRoot::Hkcu, "Software"),
        (RegRoot::Hklm, "SOFTWARE"),
        (RegRoot::Hklm, "SOFTWARE\\WOW6432Node"),
    ];

    for &(root, base) in reg_roots {
        let hive = RegKey::predef(root.hkey());
        let Ok(parent) = hive.open_subkey_with_flags(base, KEY_READ) else {
            continue;
        };
        for sub_name in parent.enum_keys().flatten() {
            if let Some(conf) = match_name(&sub_name, &tokens) {
                let full_sub = format!("{base}\\{sub_name}");
                let display = format!("{}\\{full_sub}", root.as_str());
                if seen_paths.insert(display.to_lowercase()) {
                    traces.push(AppTrace {
                        id: 0,
                        kind: TraceKind::RegistryKey,
                        confidence: conf,
                        path: full_sub,
                        size_bytes: 0,
                        selected: conf == TraceConfidence::High,
                        reg_root: Some(root),
                    });
                }
            } else if let Ok(child) = parent.open_subkey_with_flags(&sub_name, KEY_READ) {
                // Check Publisher\AppName subkey level
                for grand in child.enum_keys().flatten() {
                    if let Some(conf) = match_name(&grand, &tokens) {
                        let full_sub = format!("{base}\\{sub_name}\\{grand}");
                        let display = format!("{}\\{full_sub}", root.as_str());
                        if seen_paths.insert(display.to_lowercase()) {
                            traces.push(AppTrace {
                                id: 0,
                                kind: TraceKind::RegistryKey,
                                confidence: conf,
                                path: full_sub,
                                size_bytes: 0,
                                selected: conf == TraceConfidence::High,
                                reg_root: Some(root),
                            });
                        }
                    }
                }
            }
        }
    }

    // Check if original Uninstall registry entry is still orphaned after uninstall
    let orig_hive = RegKey::predef(app.reg_root.hkey());
    if orig_hive
        .open_subkey_with_flags(&app.reg_subkey, KEY_READ)
        .is_ok()
    {
        let display = format!("{}\\{}", app.reg_root.as_str(), app.reg_subkey);
        if seen_paths.insert(display.to_lowercase()) {
            traces.push(AppTrace {
                id: 0,
                kind: TraceKind::RegistryKey,
                confidence: TraceConfidence::High,
                path: app.reg_subkey.clone(),
                size_bytes: 0,
                selected: true,
                reg_root: Some(app.reg_root),
            });
        }
    }

    // 5. Scheduled Tasks leftovers (C:\Windows\System32\Tasks)
    let tasks_dir = PathBuf::from(r"C:\Windows\System32\Tasks");
    if tasks_dir.exists() {
        for entry in WalkDir::new(&tasks_dir).max_depth(2).into_iter().flatten() {
            let p = entry.path();
            if p.is_file() {
                let name = entry.file_name().to_string_lossy();
                if let Some(conf) = match_name(&name, &tokens) {
                    let lower = p.to_string_lossy().to_lowercase();
                    if seen_paths.insert(lower) {
                        traces.push(AppTrace {
                            id: 0,
                            kind: TraceKind::ScheduledTask,
                            confidence: conf,
                            path: p.to_string_lossy().to_string(),
                            size_bytes: 0,
                            selected: false, // Tasks unchecked by default for extra safety
                            reg_root: None,
                        });
                    }
                }
            }
        }
    }

    // 6. Services pointing into the app's install location
    if !app.install_location.is_empty() {
        let loc_lower = app.install_location.to_lowercase();
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(svcs) =
            hklm.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Services", KEY_READ)
        {
            for svc_name in svcs.enum_keys().flatten() {
                if let Ok(sk) = svcs.open_subkey_with_flags(&svc_name, KEY_READ) {
                    let img: String = sk.get_value("ImagePath").unwrap_or_default();
                    if !img.is_empty() && img.to_lowercase().contains(&loc_lower) {
                        let sub = format!("SYSTEM\\CurrentControlSet\\Services\\{svc_name}");
                        if seen_paths.insert(format!("hklm\\{sub}").to_lowercase()) {
                            traces.push(AppTrace {
                                id: 0,
                                kind: TraceKind::Service,
                                confidence: TraceConfidence::High,
                                path: sub,
                                size_bytes: 0,
                                selected: false,
                                reg_root: Some(RegRoot::Hklm),
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort High confidence first, then by size descending
    traces.sort_by(|a, b| {
        let ca = if a.confidence == TraceConfidence::High { 0 } else { 1 };
        let cb = if b.confidence == TraceConfidence::High { 0 } else { 1 };
        ca.cmp(&cb).then_with(|| b.size_bytes.cmp(&a.size_bytes))
    });
    for (idx, t) in traces.iter_mut().enumerate() {
        t.id = idx;
    }

    traces
}

fn is_safe_install_dir(path: &Path) -> bool {
    let s = path.to_string_lossy().to_lowercase();
    let trimmed = s.trim_end_matches('\\');
    // Never allow deleting root drives or top-level OS/user directories
    if trimmed.len() <= 3 {
        return false;
    }
    let forbidden_exact = [
        r"c:\windows",
        r"c:\windows\system32",
        r"c:\program files",
        r"c:\program files (x86)",
        r"c:\programdata",
        r"c:\users",
    ];
    if forbidden_exact.contains(&trimmed) {
        return false;
    }
    if trimmed.starts_with(r"c:\windows\") && !trimmed.starts_with(r"c:\windows\temp\") {
        return false;
    }
    true
}

/// Safely removes all selected leftover traces (files, folders, shortcuts, registry keys, tasks).
pub fn delete_selected_traces(traces: &[AppTrace]) -> TraceDeleteReport {
    let mut report = TraceDeleteReport::default();

    for trace in traces.iter().filter(|t| t.selected) {
        let res: Result<(), String> = match trace.kind {
            TraceKind::Directory | TraceKind::Shortcut | TraceKind::File => {
                let p = Path::new(&trace.path);
                if !p.exists() {
                    Ok(())
                } else if p.is_dir() {
                    fs::remove_dir_all(p).map_err(|e| e.to_string())
                } else {
                    fs::remove_file(p).map_err(|e| e.to_string())
                }
            }
            TraceKind::RegistryKey | TraceKind::Service => {
                if let Some(root) = trace.reg_root {
                    let hive = RegKey::predef(root.hkey());
                    hive.delete_subkey_all(&trace.path)
                        .map_err(|e| e.to_string())
                } else {
                    Err("Missing registry root".into())
                }
            }
            TraceKind::ScheduledTask => {
                let p = Path::new(&trace.path);
                if p.exists() {
                    fs::remove_file(p).map_err(|e| e.to_string())
                } else {
                    Ok(())
                }
            }
        };

        if res.is_ok() {
            report.deleted_count += 1;
            report.freed_bytes += trace.size_bytes;
        } else {
            report.failed_count += 1;
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denylist_blocks_system_and_generic_tokens() {
        assert!(!is_safe_match_token("Windows"));
        assert!(!is_safe_match_token("Microsoft"));
        assert!(!is_safe_match_token("Google"));
        assert!(!is_safe_match_token("app"));
        assert!(!is_safe_match_token("123"));
        assert!(is_safe_match_token("Visual Studio Code"));
        assert!(is_safe_match_token("ShareX"));
    }

    #[test]
    fn safe_install_dir_blocks_os_roots() {
        assert!(!is_safe_install_dir(Path::new(r"C:\")));
        assert!(!is_safe_install_dir(Path::new(r"C:\Windows")));
        assert!(!is_safe_install_dir(Path::new(r"C:\Windows\System32")));
        assert!(!is_safe_install_dir(Path::new(r"C:\Program Files")));
        assert!(is_safe_install_dir(Path::new(r"C:\Program Files\ShareX")));
    }

    #[test]
    fn reg_date_formatting() {
        assert_eq!(format_reg_date("20261124"), "2026-11-24");
        assert_eq!(format_reg_date(""), "");
    }

    #[test]
    fn live_registry_scan_runs_fast_and_finds_apps() {
        let apps = scan_installed_apps(false);
        assert!(!apps.is_empty(), "Expected installed Windows apps in registry");
    }
}

