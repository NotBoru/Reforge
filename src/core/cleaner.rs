use crate::core::winapi::{empty_recycle_bin, flush_dns_cache, query_recycle_bin};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CleanCategoryId {
    UserTemp,
    WindowsTemp,
    BrowserCaches,
    GpuShaderCaches,
    WindowsLogsAndDumps,
    WindowsUpdateCache,
    AppCaches,
    ExplorerCaches,
    RecycleBin,
    PrefetchAndDns,
}

#[derive(Debug, Clone)]
pub struct CleanCategory {
    pub id: CleanCategoryId,
    pub name_en: &'static str,
    pub name_pt: &'static str,
    pub desc_en: &'static str,
    pub desc_pt: &'static str,
    pub paths: Vec<PathBuf>,
    pub size_bytes: u64,
    pub file_count: u64,
    pub selected: bool,
    pub requires_admin: bool,
    pub safe_default: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CleanScanResult {
    pub categories: Vec<CleanCategory>,
    pub total_bytes: u64,
    pub total_files: u64,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct CleanExecResult {
    pub freed_bytes: u64,
    pub deleted_files: u64,
    pub skipped_files: u64,
    pub elapsed_ms: u64,
}

fn push_if_exists(vec: &mut Vec<PathBuf>, p: PathBuf) {
    if p.exists() {
        vec.push(p);
    }
}

fn collect_browser_cache_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    if local.is_empty() {
        return paths;
    }
    let local_p = PathBuf::from(&local);

    // Chromium-based browsers: strictly cache subdirs ONLY (never cookies, logins, bookmarks, or history)
    let chromium_roots = [
        local_p.join(r"Google\Chrome\User Data"),
        local_p.join(r"Microsoft\Edge\User Data"),
        local_p.join(r"BraveSoftware\Brave-Browser\User Data"),
        local_p.join(r"Vivaldi\User Data"),
        local_p.join(r"Opera Software\Opera Stable"),
        local_p.join(r"Opera Software\Opera GX Stable"),
    ];

    let cache_subdirs = [
        r"Cache\Cache_Data",
        "Cache",
        "Code Cache",
        "GPUCache",
        "DawnCache",
        "GrShaderCache",
        "ShaderCache",
    ];

    for root in chromium_roots {
        if !root.exists() {
            continue;
        }
        // Check root-level caches (Opera) + profile folders (Default, Profile 1, Profile 2...)
        for sub in &cache_subdirs {
            push_if_exists(&mut paths, root.join(sub));
        }
        if let Ok(entries) = fs::read_dir(&root) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name == "Default" || name.starts_with("Profile ") {
                    let prof = e.path();
                    for sub in &cache_subdirs {
                        push_if_exists(&mut paths, prof.join(sub));
                    }
                }
            }
        }
    }

    // Firefox & Zen Browser cache2
    for gecko_root in [
        local_p.join(r"Mozilla\Firefox\Profiles"),
        local_p.join(r"zen\Profiles"),
    ] {
        if let Ok(entries) = fs::read_dir(&gecko_root) {
            for e in entries.flatten() {
                push_if_exists(&mut paths, e.path().join("cache2"));
                push_if_exists(&mut paths, e.path().join("startupCache"));
            }
        }
    }

    paths
}

fn collect_gpu_cache_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let lp = PathBuf::from(&local);
        push_if_exists(&mut paths, lp.join(r"NVIDIA\DXCache"));
        push_if_exists(&mut paths, lp.join(r"NVIDIA\GLCache"));
        push_if_exists(&mut paths, lp.join(r"AMD\DxCache"));
        push_if_exists(&mut paths, lp.join(r"AMD\GLCache"));
        push_if_exists(&mut paths, lp.join("D3DSCache"));
    }
    if let Ok(local_low) = std::env::var("USERPROFILE") {
        let ll = PathBuf::from(local_low).join(r"AppData\LocalLow");
        push_if_exists(&mut paths, ll.join(r"NVIDIA\PerDriverVersion\DXCache"));
        push_if_exists(&mut paths, ll.join(r"Intel\ShaderCache"));
    }
    if let Ok(pd) = std::env::var("PROGRAMDATA") {
        push_if_exists(&mut paths, PathBuf::from(pd).join(r"NVIDIA Corporation\NV_Cache"));
    }
    paths
}

fn collect_app_cache_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(roaming) = std::env::var("APPDATA") {
        let rp = PathBuf::from(&roaming);
        for d in ["discord", "discordcanary", "discordptb", "vesktop"] {
            push_if_exists(&mut paths, rp.join(d).join("Cache"));
            push_if_exists(&mut paths, rp.join(d).join("Code Cache"));
            push_if_exists(&mut paths, rp.join(d).join("GPUCache"));
        }
        push_if_exists(&mut paths, rp.join(r"Code\Cache"));
        push_if_exists(&mut paths, rp.join(r"Code\CachedData"));
        push_if_exists(&mut paths, rp.join(r"Code\GPUCache"));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let lp = PathBuf::from(&local);
        push_if_exists(&mut paths, lp.join(r"Steam\htmlcache"));
        push_if_exists(&mut paths, lp.join(r"Spotify\Storage"));
        push_if_exists(&mut paths, lp.join(r"Spotify\Browser\Cache"));
        push_if_exists(&mut paths, lp.join(r"CrashDumps"));
    }
    push_if_exists(&mut paths, PathBuf::from(r"C:\Program Files (x86)\Steam\appcache\httpcache"));
    paths
}

fn build_category_templates() -> Vec<CleanCategory> {
    let user_temp = vec![std::env::temp_dir()];
    let win_temp = vec![PathBuf::from(r"C:\Windows\Temp")];

    let mut logs = Vec::new();
    push_if_exists(&mut logs, PathBuf::from(r"C:\Windows\Logs"));
    push_if_exists(&mut logs, PathBuf::from(r"C:\Windows\Panther"));
    push_if_exists(&mut logs, PathBuf::from(r"C:\Windows\Minidump"));
    push_if_exists(&mut logs, PathBuf::from(r"C:\Windows\LiveKernelReports"));
    if let Ok(pd) = std::env::var("PROGRAMDATA") {
        push_if_exists(&mut logs, PathBuf::from(pd).join(r"Microsoft\Windows\WER"));
    }

    let mut win_update = Vec::new();
    push_if_exists(&mut win_update, PathBuf::from(r"C:\Windows\SoftwareDistribution\Download"));
    push_if_exists(&mut win_update, PathBuf::from(r"C:\Windows\ServiceProfiles\NetworkService\AppData\Local\Microsoft\Windows\DeliveryOptimization\Cache"));

    let mut explorer = Vec::new();
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        push_if_exists(
            &mut explorer,
            PathBuf::from(local).join(r"Microsoft\Windows\Explorer"),
        );
    }

    let mut prefetch = Vec::new();
    push_if_exists(&mut prefetch, PathBuf::from(r"C:\Windows\Prefetch"));

    vec![
        CleanCategory {
            id: CleanCategoryId::UserTemp,
            name_en: "User Temporary Files",
            name_pt: "Arquivos Temporários do Usuário",
            desc_en: "Leftover installer extracts, scratch files & session temps (%TEMP%)",
            desc_pt: "Restos de instaladores e arquivos temporários (%TEMP%)",
            paths: user_temp,
            size_bytes: 0,
            file_count: 0,
            selected: true,
            requires_admin: false,
            safe_default: true,
        },
        CleanCategory {
            id: CleanCategoryId::WindowsTemp,
            name_en: "Windows Temp Files",
            name_pt: "Temporários do Windows",
            desc_en: "System-level temporary files in C:\\Windows\\Temp",
            desc_pt: "Arquivos temporários do sistema em C:\\Windows\\Temp",
            paths: win_temp,
            size_bytes: 0,
            file_count: 0,
            selected: true,
            requires_admin: true,
            safe_default: true,
        },
        CleanCategory {
            id: CleanCategoryId::BrowserCaches,
            name_en: "Browser Caches",
            name_pt: "Cache de Navegadores",
            desc_en: "Chrome, Edge, Brave, Firefox, Opera & Vivaldi cache (keeps logins & cookies)",
            desc_pt: "Cache do Chrome, Edge, Brave, Firefox, Opera e Vivaldi (mantém logins e cookies)",
            paths: collect_browser_cache_paths(),
            size_bytes: 0,
            file_count: 0,
            selected: true,
            requires_admin: false,
            safe_default: true,
        },
        CleanCategory {
            id: CleanCategoryId::GpuShaderCaches,
            name_en: "GPU Shader Caches",
            name_pt: "Cache de Shaders da GPU",
            desc_en: "NVIDIA, AMD, Intel & DirectX compiled shader caches",
            desc_pt: "Caches de shaders compilados NVIDIA, AMD, Intel e DirectX",
            paths: collect_gpu_cache_paths(),
            size_bytes: 0,
            file_count: 0,
            selected: true,
            requires_admin: false,
            safe_default: true,
        },
        CleanCategory {
            id: CleanCategoryId::WindowsLogsAndDumps,
            name_en: "System Logs & Crash Dumps",
            name_pt: "Logs do Sistema e Crash Dumps",
            desc_en: "Windows logs, WER error reports, Panther setup logs & minidumps",
            desc_pt: "Logs do Windows, relatórios WER, Panther e arquivos minidump",
            paths: logs,
            size_bytes: 0,
            file_count: 0,
            selected: true,
            requires_admin: true,
            safe_default: true,
        },
        CleanCategory {
            id: CleanCategoryId::WindowsUpdateCache,
            name_en: "Windows Update & Delivery Cache",
            name_pt: "Cache do Windows Update",
            desc_en: "Downloaded update packages and Delivery Optimization cache",
            desc_pt: "Pacotes já baixados do Windows Update e Otimização de Entrega",
            paths: win_update,
            size_bytes: 0,
            file_count: 0,
            selected: true,
            requires_admin: true,
            safe_default: true,
        },
        CleanCategory {
            id: CleanCategoryId::AppCaches,
            name_en: "Desktop App Caches",
            name_pt: "Cache de Aplicativos",
            desc_en: "Discord, Steam, Spotify, VS Code cache & user crash dumps",
            desc_pt: "Cache do Discord, Steam, Spotify, VS Code e crash dumps",
            paths: collect_app_cache_paths(),
            size_bytes: 0,
            file_count: 0,
            selected: true,
            requires_admin: false,
            safe_default: true,
        },
        CleanCategory {
            id: CleanCategoryId::ExplorerCaches,
            name_en: "Thumbnail & Icon Cache",
            name_pt: "Cache de Miniaturas e Ícones",
            desc_en: "Windows Explorer thumbcache_*.db and iconcache_*.db files",
            desc_pt: "Arquivos thumbcache_*.db e iconcache_*.db do Explorador",
            paths: explorer,
            size_bytes: 0,
            file_count: 0,
            selected: true,
            requires_admin: false,
            safe_default: true,
        },
        CleanCategory {
            id: CleanCategoryId::RecycleBin,
            name_en: "Recycle Bin",
            name_pt: "Lixeira do Windows",
            desc_en: "Permanently empty deleted items across all local drives",
            desc_pt: "Esvazia permanentemente a Lixeira em todos os discos",
            paths: Vec::new(),
            size_bytes: 0,
            file_count: 0,
            selected: false,
            requires_admin: false,
            safe_default: false,
        },
        CleanCategory {
            id: CleanCategoryId::PrefetchAndDns,
            name_en: "Prefetch & DNS Cache",
            name_pt: "Prefetch e Cache DNS",
            desc_en: "Clears Windows Prefetch traces and flushes the local DNS resolver",
            desc_pt: "Limpa rastros do Prefetch do Windows e renova o cache DNS",
            paths: prefetch,
            size_bytes: 0,
            file_count: 0,
            selected: false,
            requires_admin: true,
            safe_default: false,
        },
    ]
}

fn should_clean_file(cat_id: CleanCategoryId, path: &Path) -> bool {
    if cat_id == CleanCategoryId::ExplorerCaches {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        return name.starts_with("thumbcache_") || name.starts_with("iconcache_");
    }
    true
}

pub fn initial_categories() -> CleanScanResult {
    CleanScanResult {
        categories: build_category_templates(),
        total_bytes: 0,
        total_files: 0,
        elapsed_ms: 0,
    }
}

fn scan_dir_stats(cat_id: CleanCategoryId, root: &Path) -> (u64, u64) {
    if !root.exists() {
        return (0, 0);
    }
    let mut bytes = 0u64;
    let mut files = 0u64;
    // Use entry.file_type() from FindNextFileW (avoids redundant GetFileAttributesW syscalls)
    // and cap depth at 8 to prevent pathological nested cache trees from slowing scans.
    for entry in WalkDir::new(root)
        .follow_links(false)
        .max_depth(8)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file() && should_clean_file(cat_id, entry.path()) {
            if let Ok(m) = entry.metadata() {
                bytes += m.len();
                files += 1;
            }
        }
    }
    (bytes, files)
}

/// Scans all junk, log, and cache categories concurrently via Rayon.
pub fn scan_system_junk(previous_selection: Option<&[(CleanCategoryId, bool)]>) -> CleanScanResult {
    let start = Instant::now();
    let mut templates = build_category_templates();

    if let Some(prev) = previous_selection {
        for cat in &mut templates {
            if let Some(&(_, sel)) = prev.iter().find(|(id, _)| *id == cat.id) {
                cat.selected = sel;
            }
        }
    }

    templates.par_iter_mut().for_each(|cat| {
        if cat.id == CleanCategoryId::RecycleBin {
            let (bytes, count) = query_recycle_bin();
            cat.size_bytes = bytes;
            cat.file_count = count;
        } else {
            let (b, f): (u64, u64) = cat
                .paths
                .par_iter()
                .map(|p| scan_dir_stats(cat.id, p))
                .reduce(|| (0, 0), |(b1, f1), (b2, f2)| (b1 + b2, f1 + f2));
            cat.size_bytes = b;
            cat.file_count = f;
        }
    });

    let total_bytes: u64 = templates.iter().map(|c| c.size_bytes).sum();
    let total_files: u64 = templates.iter().map(|c| c.file_count).sum();

    CleanScanResult {
        categories: templates,
        total_bytes,
        total_files,
        elapsed_ms: start.elapsed().as_millis() as u64,
    }
}

/// Safely cleans all selected categories in parallel, skipping locked files gracefully.
pub fn clean_selected_categories(categories: &[CleanCategory]) -> CleanExecResult {
    let start = Instant::now();

    let (freed, deleted, skipped) = categories
        .par_iter()
        .filter(|c| c.selected)
        .map(|cat| {
            let mut local_freed = 0u64;
            let mut local_del = 0u64;
            let mut local_skip = 0u64;

            if cat.id == CleanCategoryId::RecycleBin {
                let (b, c) = query_recycle_bin();
                if empty_recycle_bin().is_ok() {
                    local_freed += b;
                    local_del += c;
                }
                return (local_freed, local_del, local_skip);
            }

            if cat.id == CleanCategoryId::PrefetchAndDns {
                let _ = flush_dns_cache();
            }

            for root in &cat.paths {
                if !root.exists() {
                    continue;
                }
                let mut dirs_to_prune = Vec::new();
                for entry in WalkDir::new(root).contents_first(true).into_iter().flatten() {
                    let p = entry.path();
                    if p == root {
                        continue;
                    }
                    if p.is_file() {
                        if !should_clean_file(cat.id, p) {
                            continue;
                        }
                        let sz = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        if fs::remove_file(p).is_ok() {
                            local_freed += sz;
                            local_del += 1;
                        } else {
                            local_skip += 1;
                        }
                    } else if p.is_dir() && cat.id != CleanCategoryId::ExplorerCaches {
                        dirs_to_prune.push(p.to_path_buf());
                    }
                }
                for d in dirs_to_prune {
                    let _ = fs::remove_dir(&d);
                }
            }

            (local_freed, local_del, local_skip)
        })
        .reduce(
            || (0, 0, 0),
            |(a1, b1, c1), (a2, b2, c2)| (a1 + a2, b1 + b2, c1 + c2),
        );

    CleanExecResult {
        freed_bytes: freed,
        deleted_files: deleted,
        skipped_files: skipped,
        elapsed_ms: start.elapsed().as_millis() as u64,
    }
}
