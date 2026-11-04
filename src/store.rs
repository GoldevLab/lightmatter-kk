//! Ajustes guardados en disco, junto al proceso. Sin cuenta y sin red.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use lightmatter::fit::FitReport;

const KEEP: usize = 40;

fn dir() -> PathBuf {
    if let Ok(root) = std::env::var("RESUMA_DATA_DIR") {
        if !root.trim().is_empty() {
            return PathBuf::from(root).join("reports");
        }
    }
    PathBuf::from("data").join("reports")
}

fn new_id() -> String {
    static SEQ: AtomicU64 = AtomicU64::new(1);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("fit_{secs:x}_{n:x}")
}

pub fn valid_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("fit_") else {
        return false;
    };
    !rest.is_empty()
        && rest.len() <= 40
        && !rest.starts_with('_')
        && !rest.ends_with('_')
        && rest.contains('_')
        && rest.chars().all(|c| c.is_ascii_hexdigit() || c == '_')
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DiskReport {
    source: String,
    report: FitReport,
}

pub struct LoadedReport {
    pub source: String,
    pub report: FitReport,
}

pub fn save_report(source: &str, report: &FitReport) -> Result<String, String> {
    let folder = dir();
    fs::create_dir_all(&folder).map_err(|_| "no se pudo crear data/reports".to_string())?;
    let id = new_id();
    let body = serde_json::to_vec(&DiskReport {
        source: source.to_string(),
        report: report.clone(),
    })
    .map_err(|_| "no se pudo serializar el ajuste".to_string())?;
    let path = folder.join(format!("{id}.json"));
    fs::write(&path, body).map_err(|_| "no se pudo guardar el ajuste".to_string())?;
    prune(&folder);
    Ok(id)
}

pub fn load_report(id: &str) -> Option<LoadedReport> {
    if !valid_id(id) {
        return None;
    }
    let path = dir().join(format!("{id}.json"));
    let bytes = fs::read(path).ok()?;
    let saved: DiskReport = serde_json::from_slice(&bytes).ok()?;
    Some(LoadedReport {
        source: saved.source,
        report: saved.report,
    })
}

fn prune(folder: &std::path::Path) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
        .filter_map(|e| {
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((modified, e.path()))
        })
        .collect();
    if files.len() <= KEEP {
        return;
    }
    files.sort_by_key(|(t, _)| *t);
    let extra = files.len() - KEEP;
    for (_, path) in files.into_iter().take(extra) {
        let _ = fs::remove_file(path);
    }
}
