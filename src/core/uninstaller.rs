use crate::core::app_icons::parse_icon_spec;
use crate::core::winapi::hidden_tokio_cmd;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use walkdir::WalkDir;
use winreg::enums::*;
use winreg::{RegKey, HKEY};

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
    pub display_icon: String,
    pub uninstall_string: String,
    pub quiet_uninstall_string: String,
    pub reg_root: RegRoot,
    pub reg_subkey: String,
    pub is_msi: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AppProcess {
    pub pid: u32,
    pub exe_name: String,
    pub image_path: String,
}

#[derive(Debug, Clone)]
pub enum UninstallStage {
    Queued,
    ClosingProcesses(usize),
    RunningUninstaller,
    Verifying,
    Removed { reboot_required: bool },
    StillPresent(String),
    Failed(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UninstallProgressEvent {
    pub app_id: String,
    pub app_name: String,
    pub stage: UninstallStage,
}

#[derive(Debug, Clone, Copy)]
pub struct UninstallOptions {
    pub close_processes: bool,
    pub force_terminate_if_hung: bool,
    pub prefer_quiet: bool,
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
            let display_icon: String = sub.get_value("DisplayIcon").unwrap_or_default();
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
            let is_msi = win_inst == 1 || uninstall_string.to_lowercase().contains("msiexec");

            apps.push(InstalledApp {
                id: format!("{}\\{}", root.as_str(), reg_subkey),
                name,
                publisher: publisher.trim().to_string(),
                version: version.trim().to_string(),
                install_date: format_reg_date(&raw_date),
                estimated_size_bytes: (size_kb as u64) * 1024,
                install_location: install_location.trim().trim_matches('"').to_string(),
                display_icon: display_icon.trim().to_string(),
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

// Process discovery and shutdown helpers.

/// Critical Windows system executables that must not be terminated.
const CRITICAL_SYSTEM_PROCESSES: &[&str] = &[
    "system",
    "registry",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "svchost.exe",
    "dwm.exe",
    "explorer.exe",
    "conhost.exe",
    "cmd.exe",
    "powershell.exe",
    "pwsh.exe",
    "msiexec.exe",
    "taskmgr.exe",
    "taskhostw.exe",
    "sihost.exe",
    "ctfmon.exe",
    "audiodg.exe",
    "spoolsv.exe",
    "fontdrvhost.exe",
    "runtimebroker.exe",
    "searchhost.exe",
    "shellexperiencehost.exe",
    "startmenuexperiencehost.exe",
    "securityhealthservice.exe",
    "msmpeng.exe",
    "reforge.exe",
];

fn is_critical_system_process(exe_name_lower: &str, image_path_lower: &str) -> bool {
    if CRITICAL_SYSTEM_PROCESSES.contains(&exe_name_lower) {
        return true;
    }
    if image_path_lower.starts_with(r"c:\windows\") {
        return true;
    }
    false
}

/// Safely finds running processes genuinely owned by `app` using strict executable image path verification.
/// Never matches by loose executable name substrings; never returns Windows system processes.
pub fn find_verified_app_processes(app: &InstalledApp) -> Vec<AppProcess> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };

    let mut verified_prefixes: Vec<String> = Vec::new();
    if !app.install_location.is_empty() {
        let loc = Path::new(&app.install_location);
        if is_safe_install_dir(loc) {
            let mut norm = app.install_location.to_lowercase();
            while norm.ends_with('\\') {
                norm.pop();
            }
            norm.push('\\');
            verified_prefixes.push(norm);
        }
    }

    let mut verified_exact_exes: Vec<String> = Vec::new();
    if let Some((icon_path, _)) = parse_icon_spec(&app.display_icon) {
        if icon_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
        {
            let lower = icon_path.to_string_lossy().to_lowercase();
            if !lower.starts_with(r"c:\windows\") {
                verified_exact_exes.push(lower);
            }
        }
    }

    if verified_prefixes.is_empty() && verified_exact_exes.is_empty() {
        return Vec::new();
    }

    let self_pid = unsafe { GetCurrentProcessId() };
    let mut matched = Vec::new();

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE || snap.is_null() {
            return matched;
        }

        let mut entry: PROCESSENTRY32W = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snap, &mut entry) != 0 {
            loop {
                let pid = entry.th32ProcessID;
                if pid > 4 && pid != self_pid {
                    let end = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    let exe_name = String::from_utf16_lossy(&entry.szExeFile[..end]);
                    let exe_lower = exe_name.to_lowercase();

                    if !CRITICAL_SYSTEM_PROCESSES.contains(&exe_lower.as_str()) {
                        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
                        if !handle.is_null() {
                            let mut buf = [0u16; 1024];
                            let mut len = buf.len() as u32;
                            if QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len)
                                != 0
                            {
                                let img = String::from_utf16_lossy(&buf[..len as usize]);
                                let img_lower = img.to_lowercase();
                                if !is_critical_system_process(&exe_lower, &img_lower) {
                                    let path_match = verified_prefixes
                                        .iter()
                                        .any(|pref| img_lower.starts_with(pref))
                                        || verified_exact_exes.iter().any(|ex| img_lower == *ex);
                                    if path_match {
                                        matched.push(AppProcess {
                                            pid,
                                            exe_name,
                                            image_path: img,
                                        });
                                    }
                                }
                            }
                            CloseHandle(handle);
                        }
                    }
                }
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }

    matched
}

fn is_pid_running(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    const STILL_ACTIVE: u32 = 259;

    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return false;
        }
        let mut code: u32 = 0;
        let ok = GetExitCodeProcess(h, &mut code);
        CloseHandle(h);
        ok != 0 && code == STILL_ACTIVE
    }
}

fn request_graceful_window_close(pids: &[u32]) {
    use windows_sys::core::BOOL;
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
    };

    unsafe extern "system" fn cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let pids = &*(lparam as *const Vec<u32>);
            let mut win_pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut win_pid);
            if pids.contains(&win_pid) {
                let _ = PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
            1
        }
    }

    let owned = pids.to_vec();
    unsafe {
        EnumWindows(Some(cb), &owned as *const Vec<u32> as LPARAM);
    }
}

fn terminate_verified_pids(pids: &[u32]) {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    for &pid in pids {
        unsafe {
            let h = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if !h.is_null() {
                let _ = TerminateProcess(h, 1);
                CloseHandle(h);
            }
        }
    }
}

async fn shutdown_app_processes(procs: &[AppProcess], force_if_hung: bool) {
    if procs.is_empty() {
        return;
    }
    let pids: Vec<u32> = procs.iter().map(|p| p.pid).collect();
    request_graceful_window_close(&pids);

    // Wait up to 3.5 seconds for graceful shutdown
    let deadline = Instant::now() + Duration::from_millis(3500);
    while Instant::now() < deadline {
        if pids.iter().all(|&pid| !is_pid_running(pid)) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(180)).await;
    }

    if force_if_hung {
        let remaining: Vec<u32> = pids.into_iter().filter(|&p| is_pid_running(p)).collect();
        if !remaining.is_empty() {
            terminate_verified_pids(&remaining);
            tokio::time::sleep(Duration::from_millis(350)).await;
        }
    }
}

fn uninstall_key_still_exists(app: &InstalledApp) -> bool {
    let hive = RegKey::predef(app.reg_root.hkey());
    hive.open_subkey_with_flags(&app.reg_subkey, KEY_READ).is_ok()
}

/// Runs the native uninstaller and returns `(exit_code, reboot_required)`.
async fn execute_uninstall_command(
    app: &InstalledApp,
    prefer_quiet: bool,
) -> Result<(i32, bool), String> {
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

    let mut cmd = hidden_tokio_cmd("cmd");
    cmd.raw_arg("/C");
    cmd.raw_arg(&cmdline);
    let status = cmd
        .status()
        .await
        .map_err(|e| format!("Failed to launch uninstaller: {e}"))?;

    let code = status.code().unwrap_or(-1);
    let reboot = code == 3010 || code == 1641;
    if status.success() || reboot || code == 1605 {
        Ok((code, reboot))
    } else {
        Err(format!("Uninstaller exited with code {code}"))
    }
}

/// Full verified uninstall lifecycle for one or multiple selected applications.
/// Handles: verified process lookup -> graceful shutdown -> native uninstall -> registry verification.
pub async fn run_bulk_uninstall_lifecycle(
    apps: Vec<InstalledApp>,
    opts: UninstallOptions,
    tx: mpsc::UnboundedSender<UninstallProgressEvent>,
) {
    for app in &apps {
        let _ = tx.send(UninstallProgressEvent {
            app_id: app.id.clone(),
            app_name: app.name.clone(),
            stage: UninstallStage::Queued,
        });
    }

    for app in apps {
        // 1. Identify & close genuinely associated processes gracefully
        if opts.close_processes {
            let app_clone = app.clone();
            let procs = tokio::task::spawn_blocking(move || find_verified_app_processes(&app_clone))
                .await
                .unwrap_or_default();
            if !procs.is_empty() {
                let _ = tx.send(UninstallProgressEvent {
                    app_id: app.id.clone(),
                    app_name: app.name.clone(),
                    stage: UninstallStage::ClosingProcesses(procs.len()),
                });
                shutdown_app_processes(&procs, opts.force_terminate_if_hung).await;
            }
        }

        // 2. Run native uninstaller
        let _ = tx.send(UninstallProgressEvent {
            app_id: app.id.clone(),
            app_name: app.name.clone(),
            stage: UninstallStage::RunningUninstaller,
        });

        let exec_res = execute_uninstall_command(&app, opts.prefer_quiet).await;

        // 3. Verify whether the app was genuinely removed from Windows Registry
        let _ = tx.send(UninstallProgressEvent {
            app_id: app.id.clone(),
            app_name: app.name.clone(),
            stage: UninstallStage::Verifying,
        });
        tokio::time::sleep(Duration::from_millis(250)).await;

        let still_exists = uninstall_key_still_exists(&app);
        let final_stage = match exec_res {
            Ok((_, reboot)) => {
                if !still_exists {
                    UninstallStage::Removed {
                        reboot_required: reboot,
                    }
                } else if reboot {
                    UninstallStage::Removed {
                        reboot_required: true,
                    }
                } else {
                    UninstallStage::StillPresent(
                        "Uninstaller exited, but the app entry is still registered (canceled or background child still running).".into(),
                    )
                }
            }
            Err(err) => {
                if !still_exists {
                    UninstallStage::Removed {
                        reboot_required: false,
                    }
                } else {
                    UninstallStage::Failed(err)
                }
            }
        };

        let _ = tx.send(UninstallProgressEvent {
            app_id: app.id,
            app_name: app.name,
            stage: final_stage,
        });
    }
}

/// Force removes a broken or orphaned uninstall registry entry.
pub fn force_remove_uninstall_entry(app: &InstalledApp) -> Result<(), String> {
    let hive = RegKey::predef(app.reg_root.hkey());
    hive.delete_subkey_all(&app.reg_subkey)
        .map_err(|e| format!("Could not remove registry key (Admin may be required): {e}"))
}

// Leftover trace scanner.

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
            "(64-bit)"
                | "(32-bit)"
                | "64-bit"
                | "32-bit"
                | "x64"
                | "x86"
                | "(x64)"
                | "(x86)"
                | "(user)"
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
        .follow_links(false)
        .max_depth(6)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

pub fn deep_scan_traces_multi(apps: &[InstalledApp]) -> Vec<AppTrace> {
    let mut all = Vec::new();
    let mut seen = HashSet::new();
    for app in apps {
        for trace in deep_scan_traces(app) {
            let key = if let Some(r) = trace.reg_root {
                format!("{}\\{}", r.as_str(), trace.path).to_lowercase()
            } else {
                trace.path.to_lowercase()
            };
            if seen.insert(key) {
                all.push(trace);
            }
        }
    }
    all.sort_by(|a, b| {
        let ca = if a.confidence == TraceConfidence::High { 0 } else { 1 };
        let cb = if b.confidence == TraceConfidence::High { 0 } else { 1 };
        ca.cmp(&cb).then_with(|| b.size_bytes.cmp(&a.size_bytes))
    });
    for (idx, t) in all.iter_mut().enumerate() {
        t.id = idx;
    }
    all
}

/// Parallel leftover trace scan across filesystem, registry, shortcuts, tasks, and services.
pub fn deep_scan_traces(app: &InstalledApp) -> Vec<AppTrace> {
    let tokens = build_search_tokens(app);
    let mut traces: Vec<AppTrace> = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

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
        let covered = seen_paths
            .iter()
            .any(|p| lower == *p || lower.starts_with(&format!("{p}\\")));
        if !covered && seen_paths.insert(lower) {
            traces.push(hit);
        }
    }

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

    if uninstall_key_still_exists(app) {
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

    let tasks_dir = PathBuf::from(r"C:\Windows\System32\Tasks");
    if tasks_dir.exists() {
        for entry in WalkDir::new(&tasks_dir).max_depth(2).into_iter().flatten() {
            let p = entry.path();
            if entry.file_type().is_file() {
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
                            selected: false,
                            reg_root: None,
                        });
                    }
                }
            }
        }
    }

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
    fn critical_system_processes_are_strictly_protected() {
        assert!(is_critical_system_process("explorer.exe", r"c:\windows\explorer.exe"));
        assert!(is_critical_system_process("svchost.exe", r"c:\windows\system32\svchost.exe"));
        assert!(is_critical_system_process("reforge.exe", r"c:\tools\reforge.exe"));
        assert!(!is_critical_system_process("discord.exe", r"c:\users\u\appdata\local\discord\discord.exe"));
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
