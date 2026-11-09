//! Ajustes guardados en disco, junto al proceso. Sin cuenta y sin red.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use lightmatter::fit::{FitReport, TextFit};

const KEEP: usize = 40;
const KEEP_JOBS: usize = 40;

fn root_dir() -> PathBuf {
    if let Ok(root) = std::env::var("RESUMA_DATA_DIR") {
        if !root.trim().is_empty() {
            return PathBuf::from(root);
        }
    }
    PathBuf::from("data")
}

fn dir() -> PathBuf {
    root_dir().join("reports")
}

fn jobs_dir() -> PathBuf {
    root_dir().join("jobs")
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

fn new_job_id() -> String {
    static SEQ: AtomicU64 = AtomicU64::new(1);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("job_{nanos:x}_{n:x}")
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

pub fn valid_job_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("job_") else {
        return false;
    };
    (rest.len() >= 8 && rest.len() <= 60)
        && !rest.starts_with('_')
        && !rest.ends_with('_')
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DiskJob {
    id: String,
    source: String,
    options: TextFit,
    state: JobState,
    report_id: Option<String>,
    error: Option<String>,
    created_at: u64,
    updated_at: u64,
}

#[derive(serde::Serialize)]
pub struct JobStatus {
    pub id: String,
    pub state: JobState,
    pub report_id: Option<String>,
    pub error: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct FitJob {
    pub id: String,
    pub source: String,
    pub text: String,
    pub options: TextFit,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn job_path(folder: &std::path::Path, id: &str) -> PathBuf {
    folder.join(format!("{id}.json"))
}

fn job_input_path(folder: &std::path::Path, id: &str) -> PathBuf {
    folder.join(format!("{id}.input"))
}

fn read_job(id: &str) -> Option<DiskJob> {
    if !valid_job_id(id) {
        return None;
    }
    let bytes = fs::read(job_path(&jobs_dir(), id)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn write_job(job: &DiskJob) -> Result<(), String> {
    let folder = jobs_dir();
    fs::create_dir_all(&folder).map_err(|_| "no se pudo crear data/jobs".to_string())?;
    let body = serde_json::to_vec(job).map_err(|_| "no se pudo serializar el trabajo".to_string())?;
    let path = job_path(&folder, &job.id);
    let temp = folder.join(format!("{}.json.tmp", job.id));
    fs::write(&temp, body).map_err(|_| "no se pudo guardar el estado del trabajo".to_string())?;
    fs::rename(temp, path).map_err(|_| "no se pudo actualizar el estado del trabajo".to_string())
}

pub fn create_job(
    requested_id: Option<&str>,
    source: &str,
    text: &str,
    options: &TextFit,
) -> Result<(String, bool), String> {
    let id = requested_id.map(str::to_string).unwrap_or_else(new_job_id);
    if !valid_job_id(&id) {
        return Err("el identificador del trabajo no es válido".into());
    }
    let folder = jobs_dir();
    fs::create_dir_all(&folder).map_err(|_| "no se pudo crear data/jobs".to_string())?;
    if job_path(&folder, &id).exists() {
        return Ok((id, false));
    }
    fs::write(job_input_path(&folder, &id), text)
        .map_err(|_| "no se pudo guardar el espectro para el trabajo".to_string())?;
    let timestamp = now();
    let job = DiskJob {
        id: id.clone(),
        source: source.to_string(),
        options: options.clone(),
        state: JobState::Queued,
        report_id: None,
        error: None,
        created_at: timestamp,
        updated_at: timestamp,
    };
    if let Err(error) = write_job(&job) {
        let _ = fs::remove_file(job_input_path(&folder, &id));
        return Err(error);
    }
    prune_jobs();
    Ok((id, true))
}

pub fn load_fit_job(id: &str) -> Option<FitJob> {
    let job = read_job(id)?;
    let text = fs::read_to_string(job_input_path(&jobs_dir(), id)).ok()?;
    Some(FitJob {
        id: job.id,
        source: job.source,
        text,
        options: job.options,
    })
}

pub fn job_status(id: &str) -> Option<JobStatus> {
    let job = read_job(id)?;
    Some(JobStatus {
        id: job.id,
        state: job.state,
        report_id: job.report_id,
        error: job.error,
        created_at: job.created_at,
        updated_at: job.updated_at,
    })
}

fn update_job(id: &str, state: JobState, report_id: Option<String>, error: Option<String>) -> Result<(), String> {
    let mut job = read_job(id).ok_or_else(|| "no se encontró el trabajo".to_string())?;
    job.state = state;
    job.report_id = report_id;
    job.error = error;
    job.updated_at = now();
    write_job(&job)?;
    if matches!(state, JobState::Completed | JobState::Failed) {
        let _ = fs::remove_file(job_input_path(&jobs_dir(), id));
        prune_jobs();
    }
    Ok(())
}

pub fn mark_job_running(id: &str) -> Result<(), String> {
    update_job(id, JobState::Running, None, None)
}

pub fn mark_job_completed(id: &str, report_id: &str) -> Result<(), String> {
    update_job(id, JobState::Completed, Some(report_id.to_string()), None)
}

pub fn mark_job_failed(id: &str, error: &str) -> Result<(), String> {
    update_job(id, JobState::Failed, None, Some(error.to_string()))
}

pub fn recoverable_jobs() -> Vec<String> {
    let folder = jobs_dir();
    let Ok(entries) = fs::read_dir(&folder) else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    for entry in entries.flatten() {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Ok(bytes) = fs::read(entry.path()) else { continue };
        let Ok(mut job) = serde_json::from_slice::<DiskJob>(&bytes) else { continue };
        if matches!(job.state, JobState::Queued | JobState::Running) {
            if job.state == JobState::Running {
                job.state = JobState::Queued;
                job.updated_at = now();
                let _ = write_job(&job);
            }
            ids.push(job.id);
        }
    }
    ids
}

fn prune_jobs() {
    let folder = jobs_dir();
    let Ok(entries) = fs::read_dir(&folder) else { return };
    let mut completed = entries
        .flatten()
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
        .filter_map(|entry| {
            let bytes = fs::read(entry.path()).ok()?;
            let job: DiskJob = serde_json::from_slice(&bytes).ok()?;
            matches!(job.state, JobState::Completed | JobState::Failed)
                .then_some((job.updated_at, job.id))
        })
        .collect::<Vec<_>>();
    if completed.len() <= KEEP_JOBS {
        return;
    }
    completed.sort_by_key(|(updated_at, _)| *updated_at);
    let remove_count = completed.len().saturating_sub(KEEP_JOBS);
    for (_, id) in completed.into_iter().take(remove_count) {
        let _ = fs::remove_file(job_path(&folder, &id));
        let _ = fs::remove_file(job_input_path(&folder, &id));
    }
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
