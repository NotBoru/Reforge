use crate::core::catalog::{CatalogApp, DirectSource, InstallerKind};
use crate::core::winapi::hidden_tokio_cmd;
use futures_util::StreamExt;
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, RANGE, USER_AGENT};
use reqwest::Client;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::{mpsc, Mutex, Semaphore};

#[derive(Debug, Clone)]
pub enum InstallStage {
    Queued,
    Resolving,
    Downloading {
        downloaded: u64,
        total: u64,
        speed_bps: f64,
    },
    Installing {
        method: &'static str,
    },
    Done {
        elapsed_ms: u64,
        method: &'static str,
    },
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct InstallEvent {
    pub app_id: String,
    pub app_name: String,
    pub stage: InstallStage,
}

#[derive(Debug, Clone)]
pub struct InstallJob {
    pub id: String,
    pub name: String,
    pub winget_id: String,
    pub direct: Option<DirectSource>,
}

impl From<&CatalogApp> for InstallJob {
    fn from(app: &CatalogApp) -> Self {
        Self {
            id: app.id.to_string(),
            name: app.name.to_string(),
            winget_id: app.winget_id.to_string(),
            direct: app.direct,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WingetSearchResult {
    pub name: String,
    pub id: String,
    pub version: String,
}

#[derive(Deserialize)]
struct GhRelease {
    assets: Vec<GhAsset>,
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

fn build_http_client() -> Result<Client, String> {
    Client::builder()
        .user_agent("Reforge/1.0 (+https://github.com/reforge-win/reforge)")
        .connect_timeout(Duration::from_secs(12))
        .timeout(Duration::from_secs(600))
        .pool_max_idle_per_host(16)
        .tcp_nodelay(true)
        .build()
        .map_err(|e| e.to_string())
}

async fn resolve_direct_url(client: &Client, src: DirectSource) -> Result<(String, InstallerKind, &'static [&'static str]), String> {
    match src {
        DirectSource::StaticUrl {
            url,
            kind,
            silent_args,
        } => Ok((url.to_string(), kind, silent_args)),
        DirectSource::GitHubRelease {
            repo,
            asset_contains,
            asset_excludes,
            asset_ends_with,
            kind,
            silent_args,
        } => {
            let api = format!("https://api.github.com/repos/{repo}/releases/latest");
            let resp = client
                .get(&api)
                .header(USER_AGENT, "Reforge/1.0")
                .send()
                .await
                .map_err(|e| format!("GitHub API request failed: {e}"))?;
            if !resp.status().is_success() {
                return Err(format!("GitHub API HTTP {}", resp.status()));
            }
            let release: GhRelease = resp
                .json()
                .await
                .map_err(|e| format!("GitHub release parse failed: {e}"))?;
            let asset = release
                .assets
                .into_iter()
                .find(|a| {
                    let lower = a.name.to_lowercase();
                    lower.ends_with(&asset_ends_with.to_lowercase())
                        && asset_contains
                            .iter()
                            .all(|needle| lower.contains(&needle.to_lowercase()))
                        && !asset_excludes
                            .iter()
                            .any(|excl| lower.contains(&excl.to_lowercase()))
                })
                .ok_or_else(|| format!("No matching asset in {repo}"))?;
            Ok((asset.browser_download_url, kind, silent_args))
        }
    }
}

/// Downloads a file at maximum speed using multi-connection HTTP Range streaming (up to 6 segments)
/// when supported by the server, falling back transparently to single-stream async I/O.
async fn fast_download(
    client: &Client,
    url: &str,
    dest: &Path,
    app_id: &str,
    app_name: &str,
    tx: &mpsc::UnboundedSender<InstallEvent>,
) -> Result<(), String> {
    let head = client.head(url).send().await;
    let (final_url, total_size, supports_ranges) = match head {
        Ok(resp) if resp.status().is_success() => {
            let final_url = resp.url().to_string();
            let len = resp
                .headers()
                .get(CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let ranges = resp
                .headers()
                .get(ACCEPT_RANGES)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.eq_ignore_ascii_case("bytes"))
                .unwrap_or(false);
            (final_url, len, ranges)
        }
        _ => (url.to_string(), 0, false),
    };

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }

    // Use 6 concurrent HTTP range streams for files >= 4 MB that support Range requests
    const MIN_SEGMENTED_BYTES: u64 = 4 * 1024 * 1024;
    const SEGMENTS: u64 = 6;

    if supports_ranges && total_size >= MIN_SEGMENTED_BYTES {
        let file = File::create(dest).await.map_err(|e| e.to_string())?;
        file.set_len(total_size).await.map_err(|e| e.to_string())?;
        drop(file);

        let downloaded = Arc::new(AtomicU64::new(0));
        let start = Instant::now();
        let chunk_size = total_size / SEGMENTS;

        // Progress reporter task
        let dl_progress = Arc::clone(&downloaded);
        let tx_prog = tx.clone();
        let aid = app_id.to_string();
        let aname = app_name.to_string();
        let prog_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(120));
            loop {
                interval.tick().await;
                let done = dl_progress.load(Ordering::Relaxed);
                let elapsed = start.elapsed().as_secs_f64().max(0.05);
                let _ = tx_prog.send(InstallEvent {
                    app_id: aid.clone(),
                    app_name: aname.clone(),
                    stage: InstallStage::Downloading {
                        downloaded: done,
                        total: total_size,
                        speed_bps: done as f64 / elapsed,
                    },
                });
                if done >= total_size {
                    break;
                }
            }
        });

        let mut handles = Vec::with_capacity(SEGMENTS as usize);
        for seg in 0..SEGMENTS {
            let range_start = seg * chunk_size;
            let range_end = if seg == SEGMENTS - 1 {
                total_size - 1
            } else {
                (seg + 1) * chunk_size - 1
            };
            let c = client.clone();
            let u = final_url.clone();
            let p = dest.to_path_buf();
            let counter = Arc::clone(&downloaded);
            handles.push(tokio::spawn(async move {
                let resp = c
                    .get(&u)
                    .header(RANGE, format!("bytes={range_start}-{range_end}"))
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                if !resp.status().is_success() && resp.status().as_u16() != 206 {
                    return Err(format!("Range HTTP {}", resp.status()));
                }
                let mut f = OpenOptions::new()
                    .write(true)
                    .open(&p)
                    .await
                    .map_err(|e| e.to_string())?;
                f.seek(SeekFrom::Start(range_start))
                    .await
                    .map_err(|e| e.to_string())?;
                let mut stream = resp.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let bytes = chunk.map_err(|e| e.to_string())?;
                    f.write_all(&bytes).await.map_err(|e| e.to_string())?;
                    counter.fetch_add(bytes.len() as u64, Ordering::Relaxed);
                }
                f.flush().await.map_err(|e| e.to_string())?;
                Ok::<(), String>(())
            }));
        }

        let mut seg_err = None;
        for h in handles {
            match h.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => seg_err = Some(e),
                Err(e) => seg_err = Some(e.to_string()),
            }
        }
        prog_handle.abort();

        if let Some(e) = seg_err {
            let _ = fs::remove_file(dest).await;
            return Err(e);
        }
        let elapsed = start.elapsed().as_secs_f64().max(0.05);
        let _ = tx.send(InstallEvent {
            app_id: app_id.to_string(),
            app_name: app_name.to_string(),
            stage: InstallStage::Downloading {
                downloaded: total_size,
                total: total_size,
                speed_bps: total_size as f64 / elapsed,
            },
        });
        return Ok(());
    }

    // Fallback: high-throughput single async stream
    let resp = client
        .get(&final_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(total_size);
    let mut file = File::create(dest).await.map_err(|e| e.to_string())?;
    let mut stream = resp.bytes_stream();
    let mut done: u64 = 0;
    let start = Instant::now();
    let mut last_emit = Instant::now();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| e.to_string())?;
        file.write_all(&bytes).await.map_err(|e| e.to_string())?;
        done += bytes.len() as u64;
        if last_emit.elapsed() >= Duration::from_millis(120) {
            last_emit = Instant::now();
            let elapsed = start.elapsed().as_secs_f64().max(0.05);
            let _ = tx.send(InstallEvent {
                app_id: app_id.to_string(),
                app_name: app_name.to_string(),
                stage: InstallStage::Downloading {
                    downloaded: done,
                    total,
                    speed_bps: done as f64 / elapsed,
                },
            });
        }
    }
    file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn run_local_installer(
    file_path: &Path,
    kind: InstallerKind,
    silent_args: &[&str],
    msi_lock: &Mutex<()>,
) -> Result<(), String> {
    let status = match kind {
        InstallerKind::Msi => {
            // Windows Installer only permits one MSI execution at a time (prevents Error 1618)
            let _guard = msi_lock.lock().await;
            let mut cmd = hidden_tokio_cmd("msiexec");
            cmd.arg("/i").arg(file_path);
            for arg in silent_args {
                cmd.arg(arg);
            }
            cmd.status().await.map_err(|e| e.to_string())?
        }
        InstallerKind::Exe => {
            let mut cmd = hidden_tokio_cmd(&file_path.to_string_lossy());
            for arg in silent_args {
                cmd.arg(arg);
            }
            cmd.status().await.map_err(|e| e.to_string())?
        }
    };

    // Exit codes 0 (success) and 3010 (success, reboot recommended) are both clean installs
    let code = status.code().unwrap_or(-1);
    if status.success() || code == 3010 || code == 1641 {
        Ok(())
    } else {
        Err(format!("Installer exited with code {code}"))
    }
}

async fn run_winget_install(winget_id: &str) -> Result<(), String> {
    let output = hidden_tokio_cmd("winget")
        .args([
            "install",
            "--id",
            winget_id,
            "-e",
            "--silent",
            "--accept-package-agreements",
            "--accept-source-agreements",
            "--disable-interactivity",
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to invoke winget: {e}"))?;

    let code = output.status.code().unwrap_or(-1);
    // 0 = OK, -1978335189 (0x8A15002B) = already installed & up to date
    if output.status.success() || code == -1978335189 {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = if !stderr.trim().is_empty() {
            stderr.trim().lines().last().unwrap_or("winget error").to_string()
        } else {
            stdout
                .trim()
                .lines()
                .rev()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("winget install failed")
                .to_string()
        };
        Err(msg)
    }
}

pub fn temp_download_dir() -> PathBuf {
    std::env::temp_dir().join("Reforge").join("dl")
}

/// Runs a batch of installation jobs concurrently using Reforge's dual Direct-First + Winget-Fallback pipeline.
pub async fn run_install_batch(
    jobs: Vec<InstallJob>,
    max_concurrent: usize,
    auto_cleanup: bool,
    tx: mpsc::UnboundedSender<InstallEvent>,
) {
    let client = match build_http_client() {
        Ok(c) => Arc::new(c),
        Err(e) => {
            for j in jobs {
                let _ = tx.send(InstallEvent {
                    app_id: j.id,
                    app_name: j.name,
                    stage: InstallStage::Failed(e.clone()),
                });
            }
            return;
        }
    };

    let sem = Arc::new(Semaphore::new(max_concurrent.clamp(1, 8)));
    let msi_lock = Arc::new(Mutex::new(()));
    let dl_root = temp_download_dir();
    let _ = fs::create_dir_all(&dl_root).await;

    for job in &jobs {
        let _ = tx.send(InstallEvent {
            app_id: job.id.clone(),
            app_name: job.name.clone(),
            stage: InstallStage::Queued,
        });
    }

    let mut handles = Vec::with_capacity(jobs.len());
    for job in jobs {
        let permit_sem = Arc::clone(&sem);
        let http = Arc::clone(&client);
        let msi = Arc::clone(&msi_lock);
        let dir = dl_root.clone();
        let tx = tx.clone();

        handles.push(tokio::spawn(async move {
            let _permit = match permit_sem.acquire_owned().await {
                Ok(p) => p,
                Err(_) => return,
            };
            let started = Instant::now();

            // 1. Try ultra-fast Direct Multi-Connection Fast-Path if defined
            if let Some(direct_src) = job.direct {
                let _ = tx.send(InstallEvent {
                    app_id: job.id.clone(),
                    app_name: job.name.clone(),
                    stage: InstallStage::Resolving,
                });

                if let Ok((url, kind, silent_args)) = resolve_direct_url(&http, direct_src).await {
                    let ext = match kind {
                        InstallerKind::Exe => "exe",
                        InstallerKind::Msi => "msi",
                    };
                    let safe_id: String = job
                        .id
                        .chars()
                        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                        .collect();
                    let dest = dir.join(format!("{safe_id}_setup.{ext}"));

                    if fast_download(&http, &url, &dest, &job.id, &job.name, &tx)
                        .await
                        .is_ok()
                    {
                        let _ = tx.send(InstallEvent {
                            app_id: job.id.clone(),
                            app_name: job.name.clone(),
                            stage: InstallStage::Installing { method: "Direct" },
                        });
                        let install_res = run_local_installer(&dest, kind, silent_args, &msi).await;
                        if auto_cleanup {
                            let _ = fs::remove_file(&dest).await;
                        }
                        if install_res.is_ok() {
                            let _ = tx.send(InstallEvent {
                                app_id: job.id.clone(),
                                app_name: job.name.clone(),
                                stage: InstallStage::Done {
                                    elapsed_ms: started.elapsed().as_millis() as u64,
                                    method: "Direct",
                                },
                            });
                            return;
                        }
                    }
                }
            }

            // 2. Winget fast fallback (or primary for non-direct apps)
            let _ = tx.send(InstallEvent {
                app_id: job.id.clone(),
                app_name: job.name.clone(),
                stage: InstallStage::Installing { method: "Winget" },
            });

            match run_winget_install(&job.winget_id).await {
                Ok(()) => {
                    let _ = tx.send(InstallEvent {
                        app_id: job.id,
                        app_name: job.name,
                        stage: InstallStage::Done {
                            elapsed_ms: started.elapsed().as_millis() as u64,
                            method: "Winget",
                        },
                    });
                }
                Err(err) => {
                    let _ = tx.send(InstallEvent {
                        app_id: job.id,
                        app_name: job.name,
                        stage: InstallStage::Failed(err),
                    });
                }
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    if auto_cleanup {
        let _ = fs::remove_dir_all(&dl_root).await;
    }
}

/// Live search against the entire Winget package catalog (10,000+ Windows apps).
pub async fn search_winget_catalog(query: &str) -> Result<Vec<WingetSearchResult>, String> {
    let q = query.trim();
    if q.len() < 2 {
        return Ok(Vec::new());
    }
    let output = hidden_tokio_cmd("winget")
        .args([
            "search",
            q,
            "--source",
            "winget",
            "--accept-source-agreements",
            "--count",
            "20",
        ])
        .output()
        .await
        .map_err(|e| format!("Winget unavailable: {e}"))?;

    let text = String::from_utf8_lossy(&output.stdout);
    Ok(parse_winget_search_table(&text))
}

fn parse_winget_search_table(raw: &str) -> Vec<WingetSearchResult> {
    let lines: Vec<&str> = raw
        .lines()
        .map(|l| l.split('\r').last().unwrap_or(l))
        .collect();

    let Some(sep_idx) = lines.iter().position(|l| {
        let t = l.trim();
        t.len() >= 15 && t.chars().all(|c| c == '-')
    }) else {
        return Vec::new();
    };
    if sep_idx == 0 {
        return Vec::new();
    }

    let header = lines[sep_idx - 1];
    // Find column start byte offsets using whitespace gaps in header (works across EN/PT/etc. locales!)
    let cols = detect_header_columns(header);
    if cols.len() < 3 {
        return Vec::new();
    }

    let mut out = Vec::new();
    for line in &lines[sep_idx + 1..] {
        if line.trim().is_empty() || line.starts_with('<') {
            continue;
        }
        let name = slice_col(line, cols[0], Some(cols[1]));
        let id = slice_col(line, cols[1], Some(cols[2]));
        let version = slice_col(line, cols[2], cols.get(3).copied());
        if !name.is_empty() && !id.is_empty() && id.contains('.') {
            out.push(WingetSearchResult { name, id, version });
        }
    }
    out
}

pub fn detect_header_columns(header: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let chars: Vec<(usize, char)> = header.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && chars[i].1.is_whitespace() {
            i += 1;
        }
        if i < chars.len() {
            starts.push(i); // character index, not byte index (safe for UTF-8!)
            while i < chars.len() && !chars[i].1.is_whitespace() {
                i += 1;
            }
        }
    }
    starts
}

pub fn slice_col(line: &str, start_char: usize, end_char: Option<usize>) -> String {
    let chars: Vec<char> = line.chars().collect();
    if start_char >= chars.len() {
        return String::new();
    }
    let end = end_char.unwrap_or(chars.len()).min(chars.len());
    chars[start_char..end].iter().collect::<String>().trim().to_string()
}

/// Bootstraps or repairs Winget (Microsoft.DesktopAppInstaller) on a freshly formatted PC
/// directly from official Microsoft GitHub Releases without needing Microsoft Store.
pub async fn bootstrap_winget() -> Result<String, String> {
    let client = build_http_client()?;
    let resp = client
        .get("https://api.github.com/repos/microsoft/winget-cli/releases/latest")
        .header(USER_AGENT, "Reforge/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to reach GitHub: {e}"))?;
    let release: GhRelease = resp.json().await.map_err(|e| e.to_string())?;

    let bundle = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".msixbundle"))
        .ok_or_else(|| "Could not find AppInstaller .msixbundle asset.".to_string())?;

    let tmp_dir = std::env::temp_dir().join("Reforge").join("winget_boot");
    fs::create_dir_all(&tmp_dir).await.map_err(|e| e.to_string())?;
    let bundle_path = tmp_dir.join(&bundle.name);

    let bytes = client
        .get(&bundle.browser_download_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;
    fs::write(&bundle_path, &bytes).await.map_err(|e| e.to_string())?;

    let ps = format!(
        "Add-AppxPackage -Path '{}' -ForceApplicationShutdown -ErrorAction Stop",
        bundle_path.to_string_lossy().replace('\'', "''")
    );
    let out = hidden_tokio_cmd("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    let _ = fs::remove_dir_all(&tmp_dir).await;

    if out.status.success() {
        Ok("Winget (App Installer) installed successfully.".into())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() {
            "Add-AppxPackage failed (try installing VCLibs or running as Admin).".into()
        } else {
            err.lines().next().unwrap_or("Winget install failed").to_string()
        })
    }
}
