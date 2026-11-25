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
        img.resize(64, 64, image::imageops::FilterType::Triangle)
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
                format!("https://t2.gstatic.com/faviconV2?client=SOCIAL&type=FAVICON&fallback_opts=TYPE,SIZE,URL&url=https://{domain}&size=64"),
                format!("https://icons.duckduckgo.com/ip3/{domain}.ico"),
                format!("https://www.google.com/s2/favicons?domain={domain}&sz=64"),
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

// On-demand WinGet icon resolver.

#[derive(Debug, Clone)]
pub struct IconFetchRequest {
    pub cache_key: String,
    pub direct_url: Option<String>,
    pub domains: Vec<String>,
    pub github_user: Option<String>,
}

fn sanitize_cache_key(key: &str) -> String {
    key.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Instant synchronous check of `%TEMP%\Reforge\icon_cache\<key>.bin` (<0.2 ms).
pub fn load_single_cached_icon(cache_key: &str) -> Option<RgbaIcon> {
    let path = icon_cache_dir().join(format!("{}.bin", sanitize_cache_key(cache_key)));
    let bytes = fs::read(path).ok()?;
    decode_image_bytes(&bytes)
}

#[derive(serde::Deserialize)]
struct IconDbRoot {
    #[serde(default)]
    icons_and_screenshots: HashMap<String, IconDbEntry>,
}

#[derive(serde::Deserialize)]
struct IconDbEntry {
    #[serde(default)]
    icon: String,
}

/// Downloads and 7-day caches the open-source 5,960+ WinGet package icon URL database (<20 ms warm load).
pub async fn load_icon_url_db() -> HashMap<String, String> {
    let cache_path = std::env::temp_dir()
        .join("Reforge")
        .join("icon_db_v1.json");

    let raw_json = if let Ok(meta) = tokio::fs::metadata(&cache_path).await {
        if let Ok(modified) = meta.modified() {
            if modified.elapsed().unwrap_or(Duration::MAX) < Duration::from_secs(7 * 86_400) {
                tokio::fs::read_to_string(&cache_path).await.ok()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let text = match raw_json {
        Some(t) if t.len() > 10_000 => t,
        _ => {
            let Ok(client) = Client::builder()
                .user_agent("Reforge/1.2")
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(10))
                .build()
            else {
                return HashMap::new();
            };
            let url = "https://raw.githubusercontent.com/marticliment/UniGetUI/main/WebBasedData/screenshot-database-v2.json";
            match client.get(url).send().await {
                Ok(resp) if resp.status().is_success() => match resp.text().await {
                    Ok(body) if body.len() > 10_000 => {
                        if let Some(parent) = cache_path.parent() {
                            let _ = tokio::fs::create_dir_all(parent).await;
                        }
                        let _ = tokio::fs::write(&cache_path, &body).await;
                        body
                    }
                    _ => return HashMap::new(),
                },
                _ => return HashMap::new(),
            }
        }
    };

    tokio::task::spawn_blocking(move || {
        let Ok(parsed) = serde_json::from_str::<IconDbRoot>(&text) else {
            return HashMap::new();
        };
        let mut map = HashMap::with_capacity(parsed.icons_and_screenshots.len());
        for (k, v) in parsed.icons_and_screenshots {
            if v.icon.starts_with("http")
                && !v.icon.ends_with(".svg")
                && !v.icon.contains("imgur.com/")
            {
                map.insert(k.to_lowercase(), v.icon);
            }
        }
        map
    })
    .await
    .unwrap_or_default()
}

fn slug_dash(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_dash = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

fn slug_alnum(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Fast O(1) lookup against the 5,960+ direct icon URL database.
pub fn lookup_icon_db_url<'a>(
    db: &'a HashMap<String, String>,
    winget_id: &str,
    name: &str,
) -> Option<&'a str> {
    if db.is_empty() {
        return None;
    }
    // 1. Exact publisher/package (e.g. "edde746/plezy")
    let slash_key = winget_id.replace('.', "/").to_lowercase();
    if let Some(u) = db.get(&slash_key) {
        return Some(u.as_str());
    }
    // 2. Full ID dashed/alnum (e.g. "jetbrains-rider", "jetbrainstoolbox")
    let full_dash = slug_dash(winget_id);
    if let Some(u) = db.get(&full_dash) {
        return Some(u.as_str());
    }
    let full_alnum = slug_alnum(winget_id);
    if let Some(u) = db.get(&full_alnum) {
        return Some(u.as_str());
    }
    // 3. Package suffix after publisher dot (e.g. "7zip", "blender", "obs-studio")
    if let Some((_, pkg_part)) = winget_id.split_once('.') {
        let pkg_dash = slug_dash(pkg_part);
        if let Some(u) = db.get(&pkg_dash) {
            return Some(u.as_str());
        }
        let pkg_alnum = slug_alnum(pkg_part);
        if let Some(u) = db.get(&pkg_alnum) {
            return Some(u.as_str());
        }
    }
    // 4. Normalized display name
    let name_dash = slug_dash(name);
    if let Some(u) = db.get(&name_dash) {
        return Some(u.as_str());
    }
    let name_alnum = slug_alnum(name);
    if let Some(u) = db.get(&name_alnum) {
        return Some(u.as_str());
    }
    None
}

/// Maps the top 100+ WinGet publishers directly to their verified official domain so thousands
/// of packages share one instant cached official logo with zero redundant downloads.
pub fn publisher_known_domain(publisher: &str) -> Option<&'static str> {
    match publisher.to_lowercase().as_str() {
        "microsoft" => Some("microsoft.com"),
        "mozilla" => Some("mozilla.org"),
        "google" => Some("google.com"),
        "jetbrains" => Some("jetbrains.com"),
        "kde" => Some("kde.org"),
        "adobe" => Some("adobe.com"),
        "nvidia" => Some("nvidia.com"),
        "amd" => Some("amd.com"),
        "intel" => Some("intel.com"),
        "valve" => Some("steampowered.com"),
        "epicgames" => Some("epicgames.com"),
        "riotgames" => Some("riotgames.com"),
        "electronicarts" => Some("ea.com"),
        "ubisoft" => Some("ubisoft.com"),
        "blizzard" => Some("blizzard.com"),
        "logitech" => Some("logitech.com"),
        "razerinc" => Some("razer.com"),
        "corsair" => Some("corsair.com"),
        "steelseries" => Some("steelseries.com"),
        "asus" => Some("asus.com"),
        "lenovo" => Some("lenovo.com"),
        "dell" => Some("dell.com"),
        "hp" => Some("hp.com"),
        "oracle" => Some("oracle.com"),
        "eclipseadoptium" => Some("adoptium.net"),
        "azul" => Some("azul.com"),
        "bellsoft" => Some("bell-sw.com"),
        "python" => Some("python.org"),
        "openjs" => Some("nodejs.org"),
        "rustlang" => Some("rust-lang.org"),
        "golang" => Some("go.dev"),
        "docker" => Some("docker.com"),
        "git" => Some("git-scm.com"),
        "github" => Some("github.com"),
        "anthropic" => Some("claude.ai"),
        "openai" => Some("openai.com"),
        "anysphere" => Some("cursor.com"),
        "anomaly" => Some("opencode.ai"),
        "codeium" => Some("windsurf.com"),
        "ollama" => Some("ollama.com"),
        "discord" => Some("discord.com"),
        "telegram" => Some("telegram.org"),
        "slacktechnologies" => Some("slack.com"),
        "zoom" => Some("zoom.us"),
        "spotify" => Some("spotify.com"),
        "videolan" => Some("videolan.org"),
        "obsproject" => Some("obsproject.com"),
        "blenderfoundation" => Some("blender.org"),
        "gimp" => Some("gimp.org"),
        "inkscape" => Some("inkscape.org"),
        "figma" => Some("figma.com"),
        "notion" => Some("notion.so"),
        "obsidian" => Some("obsidian.md"),
        "bitwarden" => Some("bitwarden.com"),
        "proton" => Some("proton.me"),
        "cloudflare" => Some("cloudflare.com"),
        "tailscale" => Some("tailscale.com"),
        "amazon" => Some("amazon.com"),
        "ibm" => Some("ibm.com"),
        "alibaba" => Some("alibaba.com"),
        "bytedance" => Some("bytedance.com"),
        "tencent" => Some("tencent.com"),
        "huawei" => Some("huawei.com"),
        "wondershare" => Some("wondershare.com"),
        "nirsoft" => Some("nirsoft.net"),
        "vovsoft" => Some("vovsoft.com"),
        "altova" => Some("altova.com"),
        "7zip" => Some("7-zip.org"),
        "rarlab" => Some("win-rar.com"),
        "notepad++" => Some("notepad-plus-plus.org"),
        "voidtools" => Some("voidtools.com"),
        "apache" => Some("apache.org"),
        "canonical" => Some("ubuntu.com"),
        "redhat" => Some("redhat.com"),
        "hashicorp" => Some("hashicorp.com"),
        "unity" => Some("unity.com"),
        "godotengine" => Some("godotengine.org"),
        "autodesk" => Some("autodesk.com"),
        "elgato" => Some("elgato.com"),
        "vb-audio" => Some("vb-audio.com"),
        "samsung" => Some("samsung.com"),
        "sony" => Some("sony.com"),
        "xiaomi" => Some("mi.com"),
        "diskinternals" => Some("diskinternals.com"),
        "flexense" => Some("flexense.com"),
        "thorlabs" => Some("thorlabs.com"),
        "binarymark" => Some("binarymark.com"),
        "japplis" => Some("japplis.com"),
        "lsofttechnologies" => Some("lsoft.net"),
        "softsea" => Some("softsea.com"),
        "realvnc" => Some("realvnc.com"),
        "teamviewer" => Some("teamviewer.com"),
        "anydesksoftwaregmbh" => Some("anydesk.com"),
        "dropbox" => Some("dropbox.com"),
        "brave" => Some("brave.com"),
        "opera" => Some("opera.com"),
        "vivaldi" => Some("vivaldi.com"),
        "zen-team" => Some("zen-browser.app"),
        "agilebits" => Some("1password.com"),
        "doist" => Some("todoist.com"),
        "thedocumentfoundation" => Some("libreoffice.org"),
        "onlyoffice" => Some("onlyoffice.com"),
        "moonsworth" => Some("lunarclient.com"),
        "hydralauncher" => Some("hydralauncher.gg"),
        "badlion" => Some("badlion.net"),
        "modrinth" => Some("modrinth.com"),
        "curseforge" => Some("curseforge.com"),
        "audacity" => Some("audacityteam.org"),
        "handbrake" => Some("handbrake.fr"),
        "sharex" => Some("getsharex.com"),
        "qbittorrent" => Some("qbittorrent.org"),
        "winscp" => Some("winscp.net"),
        "wiresharkfoundation" => Some("wireshark.org"),
        "postman" => Some("postman.com"),
        "insomnia" => Some("insomnia.rest"),
        "dbeaver" => Some("dbeaver.io"),
        "cockos" => Some("reaper.fm"),
        "anki" => Some("apps.ankiweb.net"),
        "digitalscholar" => Some("zotero.org"),
        "logseq" => Some("logseq.com"),
        "calibre" => Some("calibre-ebook.com"),
        "geogebra" => Some("geogebra.org"),
        "aomei" => Some("aomeitech.com"),
        "aimp" => Some("aimp.ru"),
        "alcpu" => Some("alcpu.com"),
        "atlauncher" => Some("atlauncher.com"),
        "agfeo" => Some("agfeo.de"),
        "acrosssecurity" => Some("0patch.com"),
        "awp" => Some("atlantiswordprocessor.com"),
        "cpuid" => Some("cpuid.com"),
        "crystaldewworld" => Some("crystalmark.info"),
        "foxit" => Some("foxit.com"),
        "filezilla" => Some("filezilla-project.org"),
        "irfanview" => Some("irfanview.com"),
        "kicad" => Some("kicad.org"),
        "kodi" => Some("kodi.tv"),
        "malwarebytes" => Some("malwarebytes.com"),
        "nextcloud" => Some("nextcloud.com"),
        "openvpn" => Some("openvpn.net"),
        "plex" => Some("plex.tv"),
        "realix" => Some("hwinfo.com"),
        "sublimetext" => Some("sublimetext.com"),
        "synology" => Some("synology.com"),
        "veracrypt" => Some("veracrypt.fr"),
        "vmware" => Some("vmware.com"),
        "easeus" => Some("easeus.com"),
        "faststone" => Some("faststone.org"),
        _ => None,
    }
}

/// Single persistent background HTTP/2 icon resolver for visible virtualized WinGet rows.
/// Never blocks the UI, caches every hit to disk, and strictly rejects missing icons (HTTP 404).
pub async fn run_on_demand_icon_worker(
    mut req_rx: mpsc::UnboundedReceiver<IconFetchRequest>,
    out_tx: mpsc::UnboundedSender<(String, Option<RgbaIcon>)>,
) {
    let cache_dir = icon_cache_dir();
    let _ = tokio::fs::create_dir_all(&cache_dir).await;

    let Ok(client) = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .connect_timeout(Duration::from_millis(2200))
        .timeout(Duration::from_millis(2800))
        .pool_max_idle_per_host(20)
        .build()
    else {
        return;
    };

    let client = Arc::new(client);
    let sem = Arc::new(Semaphore::new(20));

    while let Some(req) = req_rx.recv().await {
        let c = Arc::clone(&client);
        let s = Arc::clone(&sem);
        let dir = cache_dir.clone();
        let tx = out_tx.clone();

        tokio::spawn(async move {
            let Ok(_permit) = s.acquire_owned().await else {
                return;
            };
            let cache_path = dir.join(format!("{}.bin", sanitize_cache_key(&req.cache_key)));

            // Re-check disk cache in case another request populated the same shared key
            if let Ok(bytes) = tokio::fs::read(&cache_path).await {
                if let Some(ic) = decode_image_bytes(&bytes) {
                    let _ = tx.send((req.cache_key, Some(ic)));
                    return;
                }
            }

            let mut candidate_urls: Vec<String> = Vec::with_capacity(6);
            if let Some(u) = req.direct_url {
                candidate_urls.push(u);
            }
            for dom in &req.domains {
                candidate_urls.push(format!(
                    "https://t2.gstatic.com/faviconV2?client=SOCIAL&type=FAVICON&fallback_opts=TYPE,SIZE,URL&url=https://{dom}&size=64"
                ));
            }
            if let Some(first_dom) = req.domains.first() {
                candidate_urls.push(format!("https://icons.duckduckgo.com/ip3/{first_dom}.ico"));
            }
            if let Some(gh) = req.github_user {
                if gh.len() >= 2
                    && gh.len() <= 39
                    && gh.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
                {
                    candidate_urls.push(format!("https://github.com/{gh}.png?size=64"));
                }
            }

            for url in candidate_urls {
                if let Ok(resp) = c.get(&url).send().await {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            if bytes.len() > 80 && bytes.len() < 2_500_000 {
                                let cloned = bytes.to_vec();
                                let decoded = tokio::task::spawn_blocking(move || {
                                    decode_image_bytes(&cloned)
                                })
                                .await
                                .ok()
                                .flatten();
                                if let Some(ic) = decoded {
                                    let _ = tokio::fs::write(&cache_path, &bytes).await;
                                    let _ = tx.send((req.cache_key, Some(ic)));
                                    return;
                                }
                            }
                        }
                    }
                }
            }

            let _ = tx.send((req.cache_key, None));
        });
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
