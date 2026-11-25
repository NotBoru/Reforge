use crate::core::catalog::catalog;
use crate::core::installer::{detect_header_columns, slice_col};
use crate::core::winapi::hidden_tokio_cmd;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};

#[derive(Debug, Clone)]
pub struct AppUpdate {
    pub name: String,
    pub id: String,
    pub current_version: String,
    pub available_version: String,
    pub source: String,
    pub selected: bool,
    pub status: UpdateItemStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateItemStatus {
    Available,
    Updating,
    Updated,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct UpdateProgressEvent {
    pub id: String,
    pub status: UpdateItemStatus,
}

/// Checks for outdated applications using Winget's fast local-to-remote upgrade diff.
pub async fn check_for_updates() -> Result<Vec<AppUpdate>, String> {
    let output = hidden_tokio_cmd("winget")
        .args([
            "upgrade",
            "--accept-source-agreements",
            "--disable-interactivity",
        ])
        .output()
        .await
        .map_err(|e| format!("Winget is not available: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut updates = parse_winget_upgrade_table(&stdout);

    // Enrich source tag if matched in Reforge curated catalog
    let cat = catalog();
    for upd in &mut updates {
        if cat
            .iter()
            .any(|c| c.winget_id.eq_ignore_ascii_case(&upd.id))
        {
            upd.source = "Reforge + Winget".into();
        }
    }

    Ok(updates)
}

fn parse_winget_upgrade_table(raw: &str) -> Vec<AppUpdate> {
    let lines: Vec<&str> = raw
        .lines()
        .map(|l| l.split('\r').last().unwrap_or(l))
        .collect();

    let Some(sep_idx) = lines.iter().position(|l| {
        let t = l.trim();
        t.len() >= 20 && t.chars().all(|c| c == '-')
    }) else {
        return Vec::new();
    };
    if sep_idx == 0 {
        return Vec::new();
    }

    let header = lines[sep_idx - 1];
    let cols = detect_header_columns(header);
    if cols.len() < 4 {
        return Vec::new();
    }

    let mut items = Vec::new();
    for line in &lines[sep_idx + 1..] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Stop at summary footer ("N upgrades available." / "N atualizações disponíveis.")
        if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
            && !trimmed.contains('.')
            && trimmed.split_whitespace().count() <= 6
        {
            break;
        }
        if trimmed.starts_with("The following") || trimmed.starts_with("Os seguintes") {
            break;
        }

        let name = slice_col(line, cols[0], Some(cols[1]));
        let id = slice_col(line, cols[1], Some(cols[2]));
        let current = slice_col(line, cols[2], Some(cols[3]));
        let available = slice_col(line, cols[3], cols.get(4).copied());
        let source = cols
            .get(4)
            .map(|&s| slice_col(line, s, None))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "winget".into());

        if name.is_empty() || id.is_empty() || available.is_empty() || current == available {
            continue;
        }
        if id.contains(' ') {
            continue;
        }

        items.push(AppUpdate {
            name,
            id,
            current_version: current,
            available_version: available,
            source,
            selected: true,
            status: UpdateItemStatus::Available,
        });
    }

    items
}

/// Updates selected apps in parallel (capped safely to avoid installer mutex contention).
pub async fn run_updates_batch(
    ids: Vec<String>,
    max_concurrent: usize,
    tx: mpsc::UnboundedSender<UpdateProgressEvent>,
) {
    let sem = Arc::new(Semaphore::new(max_concurrent.clamp(1, 3)));
    let mut handles = Vec::with_capacity(ids.len());

    for id in ids {
        let sem = Arc::clone(&sem);
        let tx = tx.clone();
        handles.push(tokio::spawn(async move {
            let _permit = match sem.acquire_owned().await {
                Ok(p) => p,
                Err(_) => return,
            };
            let _ = tx.send(UpdateProgressEvent {
                id: id.clone(),
                status: UpdateItemStatus::Updating,
            });

            let output = hidden_tokio_cmd("winget")
                .args([
                    "upgrade",
                    "--id",
                    &id,
                    "-e",
                    "--silent",
                    "--accept-package-agreements",
                    "--accept-source-agreements",
                    "--disable-interactivity",
                ])
                .output()
                .await;

            let status = match output {
                Ok(out) if out.status.success() => UpdateItemStatus::Updated,
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let msg = stderr
                        .trim()
                        .lines()
                        .last()
                        .or_else(|| {
                            stdout
                                .trim()
                                .lines()
                                .rev()
                                .find(|l| !l.trim().is_empty())
                        })
                        .unwrap_or("Update failed")
                        .to_string();
                    UpdateItemStatus::Failed(msg)
                }
                Err(e) => UpdateItemStatus::Failed(e.to_string()),
            };

            let _ = tx.send(UpdateProgressEvent { id, status });
        }));
    }

    for h in handles {
        let _ = h.await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_winget_upgrade_table_en_and_pt() {
        let sample_pt = "\
Nome                     ID                       Versão   Disponível Origem\r\n\
--------------------------------------------------------------------------------\r\n\
7-Zip 24.08 (x64)        7zip.7zip                24.08    24.09      winget\r\n\
Visual Studio Code       Microsoft.VisualStudioCode 1.94.0 1.95.2     winget\r\n\
2 atualizações disponíveis.\r\n";

        let parsed = parse_winget_upgrade_table(sample_pt);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].id, "7zip.7zip");
        assert_eq!(parsed[0].current_version, "24.08");
        assert_eq!(parsed[0].available_version, "24.09");
        assert_eq!(parsed[1].name, "Visual Studio Code");
    }
}

