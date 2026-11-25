use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::process::Command as StdCommand;
use tokio::process::Command as TokioCommand;
use winreg::enums::*;
use winreg::RegKey;

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_build: String,
    pub hostname: String,
    pub cpu_threads: usize,
    pub ram_total_gb: f64,
    pub disk_free_bytes: u64,
    pub disk_total_bytes: u64,
    pub is_admin: bool,
    pub winget_available: bool,
    pub winget_version: String,
}

pub fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

pub fn hidden_std_cmd(program: &str) -> StdCommand {
    let mut cmd = StdCommand::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

pub fn hidden_tokio_cmd(program: &str) -> TokioCommand {
    let mut cmd = TokioCommand::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

pub fn is_elevated() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );
        CloseHandle(token);
        ok != 0 && elevation.TokenIsElevated != 0
    }
}

pub fn relaunch_as_admin() -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_str = exe.to_string_lossy();
    let verb = to_wide("runas");
    let file = to_wide(&exe_str);

    let res = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    if (res as isize) > 32 {
        Ok(())
    } else {
        Err("UAC elevation was cancelled or denied.".to_string())
    }
}

pub fn is_system_dark_mode() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
    ) {
        if let Ok(val) = key.get_value::<u32, _>("AppsUseLightTheme") {
            return val == 0;
        }
    }
    true
}

/// Synchronizes the native Windows 10/11 title bar with Reforge's active Dark/Light theme.
pub fn apply_dwm_titlebar_theme(dark: bool) {
    use windows_sys::core::BOOL;
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute;
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
    };

    struct Payload {
        pid: u32,
        dark: BOOL,
        caption_color: u32,
    }

    unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let data = &*(lparam as *const Payload);
            let mut win_pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut win_pid);
            if win_pid == data.pid && IsWindowVisible(hwnd) != 0 {
                // DWMWA_USE_IMMERSIVE_DARK_MODE = 20
                let _ = DwmSetWindowAttribute(
                    hwnd,
                    20,
                    &data.dark as *const BOOL as *const _,
                    std::mem::size_of::<BOOL>() as u32,
                );
                // DWMWA_CAPTION_COLOR = 35 (Win11 22H2+, safely ignored on Win10)
                let _ = DwmSetWindowAttribute(
                    hwnd,
                    35,
                    &data.caption_color as *const u32 as *const _,
                    std::mem::size_of::<u32>() as u32,
                );
            }
            1
        }
    }

    // COLORREF is 0x00BBGGRR
    let caption_color: u32 = if dark {
        0x0016_1211 // #111216 (matches sidebar/canvas obsidian)
    } else {
        0x00F5_F2F0 // #F0F2F5 (matches light sidebar)
    };

    let payload = Payload {
        pid: unsafe { GetCurrentProcessId() },
        dark: if dark { 1 } else { 0 },
        caption_color,
    };

    unsafe {
        EnumWindows(Some(enum_cb), &payload as *const Payload as LPARAM);
    }
}

pub fn query_recycle_bin() -> (u64, u64) {
    use windows_sys::Win32::UI::Shell::{SHQueryRecycleBinW, SHQUERYRBINFO};

    unsafe {
        let mut info = SHQUERYRBINFO {
            cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32,
            i64Size: 0,
            i64NumItems: 0,
        };
        if SHQueryRecycleBinW(std::ptr::null(), &mut info) == 0 {
            (info.i64Size.max(0) as u64, info.i64NumItems.max(0) as u64)
        } else {
            (0, 0)
        }
    }
}

pub fn empty_recycle_bin() -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::{
        SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND,
    };

    let hr = unsafe {
        SHEmptyRecycleBinW(
            std::ptr::null_mut(),
            std::ptr::null(),
            SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
        )
    };
    // S_OK (0) or E_UNEXPECTED (already empty on some builds)
    if hr == 0 || hr == -2147418113 {
        Ok(())
    } else {
        Err(format!("SHEmptyRecycleBinW failed (0x{:08X})", hr as u32))
    }
}

pub fn flush_dns_cache() -> Result<(), String> {
    let out = hidden_std_cmd("ipconfig")
        .args(["/flushdns"])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

pub async fn create_restore_point(label: &str) -> Result<String, String> {
    if !is_elevated() {
        return Err("Administrator privileges are required to create a System Restore Point.".into());
    }
    let safe_label = label.replace('\'', "");
    let script = format!(
        "Enable-ComputerRestore -Drive \"$env:SystemDrive\\\" -ErrorAction SilentlyContinue; \
         Checkpoint-Computer -Description '{safe_label}' -RestorePointType 'MODIFY_SETTINGS' -ErrorAction Stop"
    );
    let output = hidden_tokio_cmd("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok("System Restore Point created.".into())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.contains("1440") || stderr.to_lowercase().contains("frequency") {
            Err("Windows allows one restore point every 24h by default (a recent restore point already exists).".into())
        } else if stderr.is_empty() {
            Err("Failed to create System Restore Point.".into())
        } else {
            Err(stderr.lines().next().unwrap_or("Restore point error").to_string())
        }
    }
}

pub fn open_folder_in_explorer(path: &str) {
    let _ = hidden_std_cmd("explorer").arg(path).spawn();
}

pub fn open_url(url: &str) {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let op = to_wide("open");
    let target = to_wide(url);
    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            op.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
    }
}

pub fn collect_system_info() -> SystemInfo {
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (os_name, os_build) = hklm
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .map(|k| {
            let prod: String = k.get_value("ProductName").unwrap_or_else(|_| "Windows".into());
            let build: String = k.get_value("CurrentBuild").unwrap_or_default();
            let ubr: u32 = k.get_value("UBR").unwrap_or(0);
            let build_num: u32 = build.parse().unwrap_or(0);
            let nice_name = if build_num >= 22000 && prod.contains("Windows 10") {
                prod.replace("Windows 10", "Windows 11")
            } else {
                prod
            };
            let full_build = if ubr > 0 {
                format!("{build}.{ubr}")
            } else {
                build
            };
            (nice_name, full_build)
        })
        .unwrap_or_else(|_| ("Windows".into(), String::new()));

    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC".into());
    let cpu_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let mut mem = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        dwMemoryLoad: 0,
        ullTotalPhys: 0,
        ullAvailPhys: 0,
        ullTotalPageFile: 0,
        ullAvailPageFile: 0,
        ullTotalVirtual: 0,
        ullAvailVirtual: 0,
        ullAvailExtendedVirtual: 0,
    };
    let ram_total_gb = unsafe {
        if GlobalMemoryStatusEx(&mut mem) != 0 {
            mem.ullTotalPhys as f64 / (1024.0 * 1024.0 * 1024.0)
        } else {
            0.0
        }
    };

    let root = to_wide("C:\\");
    let mut free_avail: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free: u64 = 0;
    unsafe {
        let _ = GetDiskFreeSpaceExW(
            root.as_ptr(),
            &mut free_avail,
            &mut total_bytes,
            &mut total_free,
        );
    }

    let (winget_available, winget_version) = match hidden_std_cmd("winget").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            (true, v)
        }
        _ => (false, String::new()),
    };

    SystemInfo {
        os_name,
        os_build,
        hostname,
        cpu_threads,
        ram_total_gb,
        disk_free_bytes: free_avail,
        disk_total_bytes: total_bytes,
        is_admin: is_elevated(),
        winget_available,
        winget_version,
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_speed(bps: f64) -> String {
    const MB: f64 = 1024.0 * 1024.0;
    const KB: f64 = 1024.0;
    if bps >= MB {
        format!("{:.1} MB/s", bps / MB)
    } else if bps >= KB {
        format!("{:.0} KB/s", bps / KB)
    } else {
        "0 KB/s".into()
    }
}
