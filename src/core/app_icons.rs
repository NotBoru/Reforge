use crate::core::winapi::to_wide;
use iced::widget::image::Handle as ImageHandle;
use rayon::prelude::*;
use reqwest::Client;
use std::collections::HashMap;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Semaphore};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct RgbaIcon {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl RgbaIcon {
    pub fn into_handle(self) -> ImageHandle {
        ImageHandle::from_rgba(self.width, self.height, self.pixels)
    }
}

pub fn icon_cache_dir() -> PathBuf {
    std::env::temp_dir().join("Reforge").join("icon_cache")
}

pub fn expand_env_vars(input: &str) -> String {
    let mut out = input.to_string();
    for (k, v) in std::env::vars() {
        let needle = format!("%{k}%");
        let lower = out.to_lowercase();
        let nlow = needle.to_lowercase();
        if let Some(pos) = lower.find(&nlow) {
            out.replace_range(pos..pos + needle.len(), &v);
        }
    }
    out
}

pub fn parse_icon_spec(raw: &str) -> Option<(PathBuf, i32)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (path_part, index) = if let Some(rest) = trimmed.strip_prefix('"') {
        if let Some(end_quote) = rest.find('"') {
            let p = &rest[..end_quote];
            let after = rest[end_quote + 1..].trim();
            let idx = after
                .strip_prefix(',')
                .and_then(|s| s.trim().parse::<i32>().ok())
                .unwrap_or(0);
            (p, idx)
        } else {
            (rest, 0)
        }
    } else if let Some(comma) = trimmed.rfind(',') {
        let maybe_idx = trimmed[comma + 1..].trim();
        if let Ok(idx) = maybe_idx.parse::<i32>() {
            (trimmed[..comma].trim(), idx)
        } else {
            (trimmed, 0)
        }
    } else {
        (trimmed, 0)
    };

    let expanded = expand_env_vars(path_part.trim().trim_matches('"'));
    let path = PathBuf::from(&expanded);
    if path.exists() {
        Some((path, index))
    } else {
        None
    }
}

/// Extracts the real executable path from a command line, resolving Squirrel `Update.exe --processStart Target.exe`.
pub fn extract_exe_from_cmdline(cmd: &str) -> Option<PathBuf> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return None;
    }
    let candidate = if let Some(rest) = trimmed.strip_prefix('"') {
        rest.split('"').next().unwrap_or("")
    } else {
        let lower = trimmed.to_lowercase();
        if let Some(pos) = lower.find(".exe") {
            &trimmed[..pos + 4]
        } else {
            trimmed.split_whitespace().next().unwrap_or("")
        }
    };
    let expanded = expand_env_vars(candidate.trim());
    let path = PathBuf::from(expanded);
    if !path.exists() {
        return None;
    }
    let fname = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Resolve Squirrel Electron launchers (e.g. Discord/Update.exe --processStart Discord.exe)
    if fname == "update.exe" {
        if let Some(idx) = trimmed.to_lowercase().find("--processstart") {
            let after = trimmed[idx + "--processstart".len()..].trim();
            let target_exe = after
                .trim_start_matches('"')
                .split(|c: char| c == '"' || c.is_whitespace())
                .next()
                .unwrap_or("");
            if !target_exe.is_empty() {
                if let Some(parent) = path.parent() {
                    // Check app-* subdirectories (Squirrel layout) or app.ico in parent
                    let ico = parent.join("app.ico");
                    if ico.exists() {
                        return Some(ico);
                    }
                    if let Ok(entries) = fs::read_dir(parent) {
                        for e in entries.flatten() {
                            let d = e.path();
                            if d.is_dir() {
                                let nested = d.join(target_exe);
                                if nested.exists() {
                                    return Some(nested);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if matches!(
        fname.as_str(),
        "msiexec.exe" | "rundll32.exe" | "cmd.exe" | "powershell.exe" | "wscript.exe"
    ) {
        return None;
    }
    Some(path)
}

fn decode_image_file(path: &Path) -> Option<RgbaIcon> {
    let bytes = fs::read(path).ok()?;
    decode_image_bytes(&bytes)
}

pub fn decode_image_bytes(bytes: &[u8]) -> Option<RgbaIcon> {
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = if img.width() > 64 || img.height() > 64 {
        img.resize_exact(64, 64, image::imageops::FilterType::Lanczos3)
            .to_rgba8()
    } else {
        img.to_rgba8()
    };
    let (w, h) = rgba.dimensions();
    if w == 0 || h == 0 {
        return None;
    }
    Some(RgbaIcon {
        width: w,
        height: h,
        pixels: rgba.into_raw(),
    })
}

pub fn extract_native_icon(path: &Path, index: i32) -> Option<RgbaIcon> {
    if !path.exists() {
        return None;
    }
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if ext.eq_ignore_ascii_case("ico") || ext.eq_ignore_ascii_case("png") {
            if let Some(decoded) = decode_image_file(path) {
                return Some(decoded);
            }
        }
    }

    let wide = to_wide(&path.to_string_lossy());
    let is_lnk = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("lnk"))
        .unwrap_or(false);

    unsafe {
        use windows_sys::Win32::UI::Shell::{
            ExtractIconExW, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, HICON};

        let mut hicon: HICON = std::ptr::null_mut();

        if !is_lnk {
            let mut small: HICON = std::ptr::null_mut();
            let count = ExtractIconExW(wide.as_ptr(), index, &mut hicon, &mut small, 1);
            if !small.is_null() {
                DestroyIcon(small);
            }
            if count == 0 {
                hicon = std::ptr::null_mut();
            }
        }

        if hicon.is_null() {
            let mut shfi: SHFILEINFOW = mem::zeroed();
            let ok = SHGetFileInfoW(
                wide.as_ptr(),
                0,
                &mut shfi,
                mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            );
            if ok != 0 && !shfi.hIcon.is_null() {
                hicon = shfi.hIcon;
            }
        }

        if hicon.is_null() {
            return None;
        }

        let res = hicon_to_rgba(hicon);
        DestroyIcon(hicon);
        res
    }
}

unsafe fn hicon_to_rgba(
    hicon: windows_sys::Win32::UI::WindowsAndMessaging::HICON,
) -> Option<RgbaIcon> {
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};

    unsafe {
        let mut info: ICONINFO = mem::zeroed();
        if GetIconInfo(hicon, &mut info) == 0 {
            return None;
        }

        let out = (|| {
            if info.hbmColor.is_null() {
                return None;
            }
            let mut bmp: BITMAP = mem::zeroed();
            if GetObjectW(
                info.hbmColor as *mut _,
                mem::size_of::<BITMAP>() as i32,
                &mut bmp as *mut _ as *mut _,
            ) == 0
            {
                return None;
            }
            let w = bmp.bmWidth;
            let h = bmp.bmHeight.abs();
            if w <= 0 || h <= 0 || w > 512 || h > 512 {
                return None;
            }

            let hdc = CreateCompatibleDC(std::ptr::null_mut());
            if hdc.is_null() {
                return None;
            }

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [mem::zeroed()],
            };

            let mut pixels = vec![0u8; (w * h * 4) as usize];
            let rows = GetDIBits(
                hdc,
                info.hbmColor,
                0,
                h as u32,
                pixels.as_mut_ptr() as *mut _,
                &mut bmi,
                DIB_RGB_COLORS,
            );
            DeleteDC(hdc);

            if rows == 0 {
                return None;
            }

            let has_alpha = pixels.chunks_exact(4).any(|px| px[3] != 0);
            if !has_alpha && !info.hbmMask.is_null() {
                apply_mono_mask(info.hbmMask, w, h, &mut pixels);
            }

            for px in pixels.chunks_exact_mut(4) {
                px.swap(0, 2);
            }

            if pixels.chunks_exact(4).all(|px| px[3] == 0) {
                return None;
            }

            Some(RgbaIcon {
                width: w as u32,
                height: h as u32,
                pixels,
            })
        })();

        if !info.hbmColor.is_null() {
            DeleteObject(info.hbmColor as *mut _);
        }
        if !info.hbmMask.is_null() {
            DeleteObject(info.hbmMask as *mut _);
        }
        out
    }
}

unsafe fn apply_mono_mask(
    hbm_mask: windows_sys::Win32::Graphics::Gdi::HBITMAP,
    w: i32,
    h: i32,
    pixels: &mut [u8],
) {
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS,
    };

    unsafe {
        let hdc = CreateCompatibleDC(std::ptr::null_mut());
        if hdc.is_null() {
            return;
        }
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 1,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [mem::zeroed()],
        };
        let stride = (((w + 31) / 32) * 4) as usize;
        let mut mask = vec![0u8; stride * (h as usize)];
        let copied = GetDIBits(
            hdc,
            hbm_mask,
            0,
            h as u32,
            mask.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        );
        DeleteDC(hdc);
        if copied == 0 {
            return;
        }
        for y in 0..h as usize {
            for x in 0..w as usize {
                let byte = mask[y * stride + (x / 8)];
                let transparent = ((byte >> (7 - (x % 8))) & 1) == 1;
                pixels[(y * (w as usize) + x) * 4 + 3] = if transparent { 0 } else { 255 };
            }
        }
    }
}

fn norm_lookup_key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Builds a fast index of Start Menu `.lnk` shortcuts for MSI/Store apps lacking DisplayIcon.
fn build_start_menu_shortcuts() -> HashMap<String, PathBuf> {
    let mut map = HashMap::new();
    let mut dirs = Vec::new();
    if let Ok(v) = std::env::var("APPDATA") {
        dirs.push(PathBuf::from(v).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    if let Ok(v) = std::env::var("PROGRAMDATA") {
        dirs.push(PathBuf::from(v).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    for root in dirs {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root).max_depth(3).into_iter().flatten() {
            let p = entry.path();
            if entry.file_type().is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("lnk"))
                    .unwrap_or(false)
            {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    let key = norm_lookup_key(stem);
                    if key.len() >= 3 {
                        map.entry(key).or_insert_with(|| p.to_path_buf());
                    }
                }
            }
        }
    }
    map
}

fn resolve_installed_icon(
    name: &str,
    display_icon: &str,
    install_location: &str,
    uninstall_string: &str,
    shortcuts: &HashMap<String, PathBuf>,
) -> Option<RgbaIcon> {
    // 1. Registry DisplayIcon
    if let Some((path, idx)) = parse_icon_spec(display_icon) {
        if let Some(ic) = extract_native_icon(&path, idx) {
            return Some(ic);
        }
    }

    // 2. Primary executable in InstallLocation
    if !install_location.is_empty() {
        let dir = Path::new(install_location);
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(dir) {
                let norm_app = name.to_lowercase();
                let mut candidates: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| {
                        p.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.eq_ignore_ascii_case("exe"))
                            .unwrap_or(false)
                    })
                    .filter(|p| {
                        let fname = p
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        !fname.contains("unins")
                            && !fname.contains("crash")
                            && !fname.contains("update")
                            && !fname.contains("helper")
                    })
                    .collect();
                candidates.sort_by_key(|p| {
                    let stem = p
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if norm_app.contains(&stem) && stem.len() >= 3 {
                        0
                    } else {
                        1
                    }
                });
                for c in candidates.into_iter().take(2) {
                    if let Some(ic) = extract_native_icon(&c, 0) {
                        return Some(ic);
                    }
                }
            }
        }
    }

    // 3. Executable path inside UninstallString
    if let Some(exe) = extract_exe_from_cmdline(uninstall_string) {
        if let Some(ic) = extract_native_icon(&exe, 0) {
            return Some(ic);
        }
    }

    // 4. Start Menu `.lnk` shortcut fallback (resolves MSI / Electron apps like CapCut & 7-Zip)
    let app_key = norm_lookup_key(name);
    if app_key.len() >= 3 {
        if let Some(lnk) = shortcuts.get(&app_key) {
            if let Some(ic) = extract_native_icon(lnk, 0) {
                return Some(ic);
            }
        }
        // Prefix match (e.g. "7zip2602x64edition" starts with "7zip" -> "7zipfilemanager")
        if let Some((_, lnk)) = shortcuts
            .iter()
            .find(|(k, _)| k.len() >= 4 && (app_key.starts_with(k.as_str()) || k.starts_with(&app_key)))
        {
            if let Some(ic) = extract_native_icon(lnk, 0) {
                return Some(ic);
            }
        }
    }

    None
}

pub fn extract_installed_icons_batch(
    apps: &[(String, String, String, String, String)],
) -> HashMap<String, RgbaIcon> {
    let shortcuts = build_start_menu_shortcuts();
    apps.par_iter()
        .filter_map(|(id, name, di, loc, uninst)| {
            resolve_installed_icon(name, di, loc, uninst, &shortcuts).map(|ic| (id.clone(), ic))
        })
        .collect()
}

pub fn extract_startup_icons_batch(items: &[(String, String)]) -> HashMap<String, RgbaIcon> {
    items
        .par_iter()
        .filter_map(|(id, cmd)| {
            extract_exe_from_cmdline(cmd)
                .and_then(|p| extract_native_icon(&p, 0))
                .map(|ic| (id.clone(), ic))
        })
        .collect()
}

/// Instant local disk cache load (<5 ms on warm runs), returning `(loaded, missing)`.
pub fn load_disk_cached_web_icons(
    entries: &[(String, String)],
) -> (HashMap<String, RgbaIcon>, Vec<(String, String)>) {
    let cache_dir = icon_cache_dir();
    let mut loaded = HashMap::new();
    let mut missing = Vec::new();

    for (key, domain) in entries {
        if domain.is_empty() {
            continue;
        }
        let safe: String = key
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        let path = cache_dir.join(format!("{safe}.bin"));
        if let Ok(bytes) = fs::read(&path) {
            if let Some(ic) = decode_image_bytes(&bytes) {
                loaded.insert(key.clone(), ic);
                continue;
            }
        }
        missing.push((key.clone(), domain.clone()));
    }

    (loaded, missing)
}

/// Streams missing official app logos concurrently into the UI as each individual icon downloads,
/// so no single slow host ever blocks the rest of the catalog.
pub async fn stream_missing_web_icons(
    missing: Vec<(String, String)>,
    tx: mpsc::UnboundedSender<(String, RgbaIcon)>,
) {
    if missing.is_empty() {
        return;
    }
    let cache_dir = icon_cache_dir();
    let _ = tokio::fs::create_dir_all(&cache_dir).await;

    let Ok(client) = Client::builder()
        .user_agent("Reforge/1.0")
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(4))
        .pool_max_idle_per_host(16)
        .build()
    else {
        return;
    };

    let client = Arc::new(client);
    let sem = Arc::new(Semaphore::new(16));
    let mut handles = Vec::with_capacity(missing.len());

    for (key, domain) in missing {
        let c = Arc::clone(&client);
        let s = Arc::clone(&sem);
        let dir = cache_dir.clone();
        let tx = tx.clone();

        handles.push(tokio::spawn(async move {
            let Ok(_permit) = s.acquire_owned().await else {
                return;
            };
            let safe: String = key
                .chars()
                .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
                .collect();
            let cache_path = dir.join(format!("{safe}.bin"));
            let urls = [
                format!("https://www.google.com/s2/favicons?domain={domain}&sz=64"),
                format!("https://icons.duckduckgo.com/ip3/{domain}.ico"),
            ];
            for url in urls {
                if let Ok(resp) = c.get(&url).send().await {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            if bytes.len() > 80 {
                                if let Some(ic) = decode_image_bytes(&bytes) {
                                    let _ = tokio::fs::write(&cache_path, &bytes).await;
                                    let _ = tx.send((key, ic));
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::uninstaller::scan_installed_apps;

    #[test]
    fn extracts_real_native_win32_icons_from_installed_apps() {
        let apps = scan_installed_apps(false);
        let sample: Vec<_> = apps
            .iter()
            .take(25)
            .map(|a| {
                (
                    a.id.clone(),
                    a.name.clone(),
                    a.display_icon.clone(),
                    a.install_location.clone(),
                    a.uninstall_string.clone(),
                )
            })
            .collect();
        let extracted = extract_installed_icons_batch(&sample);
        assert!(
            extracted.len() >= 5,
            "Expected real Win32 icons from installed apps, got {}",
            extracted.len()
        );
        for ic in extracted.values() {
            assert!(ic.width > 0 && ic.height > 0);
            assert_eq!(ic.pixels.len(), (ic.width * ic.height * 4) as usize);
        }
    }
}
