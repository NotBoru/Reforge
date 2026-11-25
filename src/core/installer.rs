use crate::core::catalog::{CatalogApp, DirectSource, InstallerKind};
use crate::core::winapi::{hidden_tokio_cmd, verify_authenticode_signature};
use futures_util::StreamExt;
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, RANGE, USER_AGENT};
use reqwest::Client;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::{mpsc, Mutex, Semaphore};

static SHARED_CLIENT: OnceLock<Client> = OnceLock::new();

fn shared_http_client() -> Result<Client, String> {
    if let Some(c) = SHARED_CLIENT.get() {
        return Ok(c.clone());
    }
    let client = Client::builder()
        .user_agent("Reforge/1.0 (+https://github.com/reforge-win/reforge)")
        .connect_timeout(Duration::from_secs(12))
        .timeout(Duration::from_secs(600))
        .pool_max_idle_per_host(16)
        .tcp_nodelay(true)
        .build()
        .map_err(|e| e.to_string())?;
    let _ = SHARED_CLIENT.set(client.clone());
    Ok(client)
}

#[derive(Debug, Clone)]
pub enum InstallStage {
    Queued,
    Resolving,
    Downloading {
        downloaded: u64,
        total: u64,
        speed_bps: f64,
    },
    Verifying,
    Installing {
        method: &'static str,
        signed: bool,
    },
    Done {
        elapsed_ms: u64,
        method: &'static str,
    },
    Cancelled,
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
    pub publisher: String,
    pub description: String,
    pub homepage_domain: String,
    pub version: String,
}

#[derive(Deserialize)]
struct WingetRunResponse {
    #[serde(rename = "Packages", default)]
    packages: Vec<WingetRunPkg>,
    #[serde(rename = "Total", default)]
    total: usize,
}

#[derive(Deserialize)]
struct WingetRunPkg {
    #[serde(rename = "Id", default)]
    id: String,
    #[serde(rename = "Versions", default)]
    versions: Vec<String>,
    #[serde(rename = "Latest")]
    latest: Option<WingetRunLatest>,
}

#[derive(Deserialize)]
struct WingetRunLatest {
    #[serde(rename = "Name", default)]
    name: String,
    #[serde(rename = "Publisher", default)]
    publisher: String,
    #[serde(rename = "Description", default)]
    description: String,
    #[serde(rename = "Homepage", default)]
    homepage: String,
}

fn extract_domain_from_url(url: &str) -> String {
    let s = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");
    s.split('/').next().unwrap_or("").to_string()
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

async fn resolve_direct_url(
    client: &Client,
    src: DirectSource,
) -> Result<(String, InstallerKind, &'static [&'static str]), String> {
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
                .map_err(|e| format!("GitHub API error: {e}"))?;
            if !resp.status().is_success() {
                return Err(format!("GitHub API HTTP {}", resp.status()));
            }
            let release: GhRelease = resp
                .json()
                .await
                .map_err(|e| format!("GitHub JSON error: {e}"))?;
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

async fn fast_download_with_retry(
    client: &Client,
    url: &str,
    dest: &Path,
    app_id: &str,
    app_name: &str,
    cancel: &AtomicBool,
    tx: &mpsc::UnboundedSender<InstallEvent>,
) -> Result<(), String> {
    let mut last_err = String::new();
    for attempt in 0..2 {
        if cancel.load(Ordering::Relaxed) {
            return Err("CANCELLED".into());
        }
        match fast_download_once(client, url, dest, app_id, app_name, cancel, tx).await {
            Ok(()) => return Ok(()),
            Err(e) if e == "CANCELLED" => return Err(e),
            Err(e) => {
                last_err = e;
                let _ = fs::remove_file(dest).await;
                if attempt == 0 {
                    tokio::time::sleep(Duration::from_millis(700)).await;
                }
            }
        }
    }
    Err(last_err)
}

async fn fast_download_once(
    client: &Client,
    url: &str,
    dest: &Path,
    app_id: &str,
    app_name: &str,
    cancel: &AtomicBool,
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

    if cancel.load(Ordering::Relaxed) {
        return Err("CANCELLED".into());
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }

    const MIN_SEGMENTED_BYTES: u64 = 4 * 1024 * 1024;
    const SEGMENTS: u64 = 6;

    if supports_ranges && total_size >= MIN_SEGMENTED_BYTES {
        let file = File::create(dest).await.map_err(|e| e.to_string())?;
        file.set_len(total_size).await.map_err(|e| e.to_string())?;
        drop(file);

        let downloaded = Arc::new(AtomicU64::new(0));
        let start = Instant::now();
        let chunk_size = total_size / SEGMENTS;

        let dl_progress = Arc::clone(&downloaded);
        let tx_prog = tx.clone();
        let aid = app_id.to_string();
        let aname = app_name.to_string();
        let prog_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
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
        let cancel_shared = Arc::new(AtomicBool::new(false));
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
            let local_cancel = Arc::clone(&cancel_shared);
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
                    if local_cancel.load(Ordering::Relaxed) {
                        return Err("CANCELLED".into());
                    }
                    let bytes = chunk.map_err(|e| e.to_string())?;
                    f.write_all(&bytes).await.map_err(|e| e.to_string())?;
                    counter.fetch_add(bytes.len() as u64, Ordering::Relaxed);
                }
                f.flush().await.map_err(|e| e.to_string())?;
                Ok::<(), String>(())
            }));
        }

        // Propagate parent cancellation while awaiting segments
        let mut seg_err = None;
        for h in handles {
            if cancel.load(Ordering::Relaxed) {
                cancel_shared.store(true, Ordering::Relaxed);
            }
            match h.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => seg_err = Some(e),
                Err(e) => seg_err = Some(e.to_string()),
            }
        }
        prog_handle.abort();

        if cancel.load(Ordering::Relaxed) {
            let _ = fs::remove_file(dest).await;
            return Err("CANCELLED".into());
        }
        if let Some(e) = seg_err {
            let _ = fs::remove_file(dest).await;
            return Err(e);
        }
        return Ok(());
    }

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
        if cancel.load(Ordering::Relaxed) {
            drop(file);
            let _ = fs::remove_file(dest).await;
            return Err("CANCELLED".into());
        }
        let bytes = chunk.map_err(|e| e.to_string())?;
        file.write_all(&bytes).await.map_err(|e| e.to_string())?;
        done += bytes.len() as u64;
        if last_emit.elapsed() >= Duration::from_millis(100) {
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
    if output.status.success() || code == -1978335189 {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = if !stderr.trim().is_empty() {
            stderr
                .trim()
                .lines()
                .last()
                .unwrap_or("winget error")
                .to_string()
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

/// Runs a batch of installation jobs concurrently with Authenticode verification,
/// controlled concurrency, automatic retry, cancellation, and Winget fallback.
pub async fn run_install_batch(
    jobs: Vec<InstallJob>,
    max_concurrent: usize,
    auto_cleanup: bool,
    cancel: Arc<AtomicBool>,
    tx: mpsc::UnboundedSender<InstallEvent>,
) {
    let client = match shared_http_client() {
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

    let sem = Arc::new(Semaphore::new(max_concurrent.clamp(1, 6)));
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
        let cancel_flag = Arc::clone(&cancel);
        let tx = tx.clone();

        handles.push(tokio::spawn(async move {
            let _permit = match permit_sem.acquire_owned().await {
                Ok(p) => p,
                Err(_) => return,
            };

            if cancel_flag.load(Ordering::Relaxed) {
                let _ = tx.send(InstallEvent {
                    app_id: job.id,
                    app_name: job.name,
                    stage: InstallStage::Cancelled,
                });
                return;
            }

            let started = Instant::now();

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

                    match fast_download_with_retry(
                        &http,
                        &url,
                        &dest,
                        &job.id,
                        &job.name,
                        &cancel_flag,
                        &tx,
                    )
                    .await
                    {
                        Ok(()) => {
                            let _ = tx.send(InstallEvent {
                                app_id: job.id.clone(),
                                app_name: job.name.clone(),
                                stage: InstallStage::Verifying,
                            });
                            let dest_verify = dest.clone();
                            let signed = tokio::task::spawn_blocking(move || {
                                verify_authenticode_signature(&dest_verify)
                            })
                            .await
                            .unwrap_or(false);

                            if cancel_flag.load(Ordering::Relaxed) {
                                if auto_cleanup {
                                    let _ = fs::remove_file(&dest).await;
                                }
                                let _ = tx.send(InstallEvent {
                                    app_id: job.id,
                                    app_name: job.name,
                                    stage: InstallStage::Cancelled,
                                });
                                return;
                            }

                            let _ = tx.send(InstallEvent {
                                app_id: job.id.clone(),
                                app_name: job.name.clone(),
                                stage: InstallStage::Installing {
                                    method: "Direct",
                                    signed,
                                },
                            });

                            let install_res =
                                run_local_installer(&dest, kind, silent_args, &msi).await;
                            if auto_cleanup {
                                let _ = fs::remove_file(&dest).await;
                            }
                            if install_res.is_ok() {
                                let _ = tx.send(InstallEvent {
                                    app_id: job.id,
                                    app_name: job.name,
                                    stage: InstallStage::Done {
                                        elapsed_ms: started.elapsed().as_millis() as u64,
                                        method: "Direct",
                                    },
                                });
                                return;
                            }
                        }
                        Err(e) if e == "CANCELLED" => {
                            let _ = tx.send(InstallEvent {
                                app_id: job.id,
                                app_name: job.name,
                                stage: InstallStage::Cancelled,
                            });
                            return;
                        }
                        Err(_) => {} // Fall through cleanly to Winget
                    }
                }
            }

            if cancel_flag.load(Ordering::Relaxed) {
                let _ = tx.send(InstallEvent {
                    app_id: job.id,
                    app_name: job.name,
                    stage: InstallStage::Cancelled,
                });
                return;
            }

            let _ = tx.send(InstallEvent {
                app_id: job.id.clone(),
                app_name: job.name.clone(),
                stage: InstallStage::Installing {
                    method: "Winget",
                    signed: true,
                },
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

fn encode_query_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn winget_index_cache_path() -> PathBuf {
    std::env::temp_dir()
        .join("Reforge")
        .join("winget_index_v1.tsv")
}

/// Loads the full 10,000–16,000+ live Microsoft WinGet package catalog with 24-hour disk cache
/// (<15 ms on warm loads, ~4 s first-time cold dump from local `winget`, with automatic fallback
/// to `api.winget.run` if WinGet CLI is not yet installed).
pub async fn load_full_winget_index() -> Result<Vec<WingetSearchResult>, String> {
    let cache_path = winget_index_cache_path();
    if let Ok(meta) = fs::metadata(&cache_path).await {
        if let Ok(modified) = meta.modified() {
            if modified.elapsed().unwrap_or(Duration::MAX) < Duration::from_secs(86_400) {
                if let Ok(raw) = fs::read_to_string(&cache_path).await {
                    let parsed = parse_cached_index_tsv(&raw);
                    if parsed.len() >= 1000 {
                        return Ok(parsed);
                    }
                }
            }
        }
    }

    // Primary: dump entire live official Microsoft WinGet catalog (16,000+ packages, always up to date)
    if let Ok(output) = hidden_tokio_cmd("winget")
        .args(["search", ".", "--source", "winget", "--accept-source-agreements"])
        .output()
        .await
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut parsed = parse_winget_search_table(&text);
            if parsed.len() >= 100 {
                enrich_with_curated_metadata(&mut parsed);
                if let Some(parent) = cache_path.parent() {
                    let _ = fs::create_dir_all(parent).await;
                }
                let tsv = serialize_index_tsv(&parsed);
                let _ = fs::write(&cache_path, tsv).await;
                return Ok(parsed);
            }
        }
    }

    // Fallback (e.g., fresh Windows ISO before WinGet bootstrap): query winget.run v2 REST API
    let (items, _) = search_winget_catalog("", 1).await?;
    Ok(items)
}

fn enrich_with_curated_metadata(items: &mut [WingetSearchResult]) {
    for it in items.iter_mut() {
        if let Some(cat) = crate::core::catalog::find_by_winget_id(&it.id) {
            if it.description.is_empty() {
                it.description = cat.desc_en.to_string();
            }
            if it.homepage_domain.is_empty() {
                it.homepage_domain = cat.domain.to_string();
            }
            if it.publisher.is_empty() || it.publisher == it.id.split('.').next().unwrap_or("") {
                it.publisher = cat.publisher.to_string();
            }
        }
    }
}

fn serialize_index_tsv(items: &[WingetSearchResult]) -> String {
    let mut out = String::with_capacity(items.len() * 64);
    for it in items {
        let clean = |s: &str| s.replace(['\t', '\r', '\n'], " ");
        out.push_str(&clean(&it.id));
        out.push('\t');
        out.push_str(&clean(&it.name));
        out.push('\t');
        out.push_str(&clean(&it.publisher));
        out.push('\t');
        out.push_str(&clean(&it.version));
        out.push('\t');
        out.push_str(&clean(&it.description));
        out.push('\t');
        out.push_str(&clean(&it.homepage_domain));
        out.push('\n');
    }
    out
}

fn parse_cached_index_tsv(raw: &str) -> Vec<WingetSearchResult> {
    let mut out = Vec::with_capacity(16_000);
    for line in raw.lines() {
        let mut parts = line.split('\t');
        let id = parts.next().unwrap_or("").to_string();
        let name = parts.next().unwrap_or("").to_string();
        let publisher = parts.next().unwrap_or("").to_string();
        let version = parts.next().unwrap_or("").to_string();
        let description = parts.next().unwrap_or("").to_string();
        let homepage_domain = parts.next().unwrap_or("").to_string();
        if !id.is_empty() && !name.is_empty() {
            out.push(WingetSearchResult {
                name,
                id,
                publisher,
                description,
                homepage_domain,
                version,
            });
        }
    }
    out
}

/// Ultra-fast live search & browse across the WinGet community repository.
/// Queries local `winget search` (up-to-date 16,000+ catalog) when querying a term, or
/// `api.winget.run/v2/packages` as a zero-CLI REST fallback.
pub async fn search_winget_catalog(
    query: &str,
    page: usize,
) -> Result<(Vec<WingetSearchResult>, usize), String> {
    let q = query.trim();

    if q.len() >= 2 {
        if let Ok(output) = hidden_tokio_cmd("winget")
            .args([
                "search",
                q,
                "--source",
                "winget",
                "--accept-source-agreements",
                "--count",
                "80",
            ])
            .output()
            .await
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut parsed = parse_winget_search_table(&text);
                if !parsed.is_empty() {
                    enrich_with_curated_metadata(&mut parsed);
                    let count = parsed.len();
                    return Ok((parsed, count));
                }
            }
        }
    }

    if let Ok(client) = shared_http_client() {
        let page_num = page.max(1);
        let url = if q.is_empty() {
            format!("https://api.winget.run/v2/packages?take=40&page={page_num}")
        } else {
            format!(
                "https://api.winget.run/v2/packages?query={}&take=40&page={page_num}",
                encode_query_component(q)
            )
        };
        let req = client.get(&url).timeout(Duration::from_secs(6));

        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(parsed) = resp.json::<WingetRunResponse>().await {
                    let results: Vec<WingetSearchResult> = parsed
                        .packages
                        .into_iter()
                        .filter_map(|p| {
                            let latest = p.latest?;
                            let version = p.versions.into_iter().next().unwrap_or_default();
                            let domain = extract_domain_from_url(&latest.homepage);
                            let name = if latest.name.is_empty() {
                                p.id.clone()
                            } else {
                                latest.name
                            };
                            Some(WingetSearchResult {
                                name,
                                id: p.id,
                                publisher: latest.publisher,
                                description: latest.description,
                                homepage_domain: domain,
                                version,
                            })
                        })
                        .collect();
                    let total = parsed.total.max(results.len());
                    return Ok((results, total));
                }
            }
        }
    }

    Ok((Vec::new(), 0))
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
    let cols = detect_header_columns(header);
    if cols.len() < 3 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(lines.len().saturating_sub(sep_idx));
    for line in &lines[sep_idx + 1..] {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('<') {
            continue;
        }
        let mut name = slice_col(line, cols[0], Some(cols[1]));
        let mut id = slice_col(line, cols[1], Some(cols[2]));
        let mut version = slice_col(line, cols[2], cols.get(3).copied());

        // Handle wide Unicode characters shifting column indices by falling back to gap splitting
        if id.is_empty() || !id.contains('.') || id.contains(' ') {
            let chunks: Vec<&str> = trimmed
                .split("  ")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if chunks.len() >= 3 && chunks[1].contains('.') && !chunks[1].contains(' ') {
                name = chunks[0].to_string();
                id = chunks[1].to_string();
                version = chunks[2].to_string();
            }
        }

        if !name.is_empty() && !id.is_empty() && id.contains('.') && !id.contains(' ') {
            let pub_part = id.split('.').next().unwrap_or("").to_string();
            out.push(WingetSearchResult {
                name,
                id,
                publisher: pub_part,
                description: String::new(),
                homepage_domain: String::new(),
                version,
            });
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
            starts.push(i);
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
    chars[start_char..end]
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
}

pub async fn bootstrap_winget() -> Result<String, String> {
    let client = shared_http_client()?;
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
            "Add-AppxPackage failed (try running as Admin).".into()
        } else {
            err.lines()
                .next()
                .unwrap_or("Winget install failed")
                .to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_query_and_extracts_homepage_domain() {
        assert_eq!(encode_query_component("lunar client+mc"), "lunar%20client%2Bmc");
        assert_eq!(
            extract_domain_from_url("https://www.lunarclient.com/download"),
            "lunarclient.com"
        );
        assert_eq!(
            extract_domain_from_url("https://hydralauncher.gg/"),
            "hydralauncher.gg"
        );
    }
}
