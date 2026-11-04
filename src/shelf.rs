//! Estante público. Cada publicación es una copia del ajuste, aparte de los 40 últimos del laboratorio.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Form, Path};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use lightmatter::fit::FitReport;
use serde::{Deserialize, Serialize};

const MAX_ITEMS: usize = 400;

static LOCK: Mutex<()> = Mutex::new(());
static HITS: Mutex<Vec<u64>> = Mutex::new(Vec::new());

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub at: u64,
    pub title: String,
    pub summary: String,
    pub measure: String,
    pub unit: String,
    pub points: usize,
    pub chisq: f64,
    pub peak: String,
    pub spark: Vec<f64>,
}

#[derive(Serialize, Deserialize)]
struct Catalog {
    items: Vec<Card>,
}

#[derive(Serialize, Deserialize)]
struct Saved {
    card: Card,
    source: String,
    report: FitReport,
}

pub struct Opened {
    pub card: Card,
    pub source: String,
    pub report: FitReport,
}

fn root() -> PathBuf {
    if let Ok(dir) = std::env::var("RESUMA_DATA_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir).join("shelf");
        }
    }
    PathBuf::from("data").join("shelf")
}

pub fn list() -> Vec<Card> {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    read_catalog().items
}

pub fn load(id: &str) -> Option<Opened> {
    if !valid_id(id) {
        return None;
    }
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bytes = fs::read(root().join(format!("{id}.json"))).ok()?;
    let saved: Saved = serde_json::from_slice(&bytes).ok()?;
    Some(Opened {
        card: saved.card,
        source: saved.source,
        report: saved.report,
    })
}

pub fn publish(source: &str, report: &FitReport, title: &str, summary: &str) -> Result<String, &'static str> {
    if too_many() {
        return Err("rate");
    }
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let folder = root();
    fs::create_dir_all(&folder).map_err(|_| "disk")?;
    let id = new_id();
    let card = Card {
        id: id.clone(),
        at: now(),
        title: title.to_string(),
        summary: summary.to_string(),
        measure: report.measure().key().to_string(),
        unit: report.options.unit.key().to_string(),
        points: report.points_used,
        chisq: report.chisq,
        peak: peak_label(report),
        spark: spark_of(report),
    };
    let body = serde_json::to_vec(&Saved {
        card: card.clone(),
        source: source.to_string(),
        report: report.clone(),
    })
    .map_err(|_| "disk")?;
    fs::write(folder.join(format!("{id}.json")), body).map_err(|_| "disk")?;
    let mut catalog = read_catalog();
    catalog.items.insert(0, card);
    while catalog.items.len() > MAX_ITEMS {
        if let Some(old) = catalog.items.pop() {
            let _ = fs::remove_file(folder.join(format!("{}.json", old.id)));
        }
    }
    write_catalog(&catalog)?;
    note_hit();
    Ok(id)
}

pub fn valid_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("pub_") else {
        return false;
    };
    !rest.is_empty()
        && rest.len() <= 40
        && rest.contains('_')
        && rest.chars().all(|c| c.is_ascii_hexdigit() || c == '_')
}

#[derive(Deserialize)]
pub struct PublishForm {
    report_id: String,
    title: String,
    summary: String,
}

pub async fn publish_form(Form(form): Form<PublishForm>) -> Response {
    let Some(loaded) = crate::store::load_report(form.report_id.trim()) else {
        return redirect("/lab");
    };
    let lang = crate::locale::current();
    let (fallback_title, fallback_summary) = lightmatter::publish::suggest(&loaded.report, lang);
    let title = clean(&form.title, 90);
    let summary = clean(&form.summary, 700);
    let title = if title.chars().count() >= 3 { title } else { fallback_title };
    let summary = if summary.chars().count() >= 12 { summary } else { fallback_summary };
    match publish(&loaded.source, &loaded.report, &title, &summary) {
        Ok(id) => redirect(&format!("/shelf/{id}")),
        Err("rate") => redirect("/shelf?e=2"),
        Err(_) => redirect("/shelf?e=1"),
    }
}

pub async fn download(Path(id): Path<String>) -> Response {
    send_pub(&id, "dat")
}

pub async fn download_as(Path((id, format)): Path<(String, String)>) -> Response {
    send_pub(&id, &format)
}

fn send_pub(id: &str, format: &str) -> Response {
    let Some(opened) = load(id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    crate::http::deliver(id, &opened.source, &opened.report, format)
}

fn read_catalog() -> Catalog {
    let path = root().join("index.json");
    fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Catalog { items: Vec::new() })
}

fn write_catalog(catalog: &Catalog) -> Result<(), &'static str> {
    let body = serde_json::to_vec(catalog).map_err(|_| "disk")?;
    fs::write(root().join("index.json"), body).map_err(|_| "disk")
}

fn new_id() -> String {
    static SEQ: AtomicU64 = AtomicU64::new(1);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("pub_{:x}_{n:x}", now())
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn too_many() -> bool {
    let now = now();
    let mut hits = HITS.lock().unwrap_or_else(|e| e.into_inner());
    hits.retain(|t| now.saturating_sub(*t) < 600);
    hits.len() >= 8
}

fn note_hit() {
    let mut hits = HITS.lock().unwrap_or_else(|e| e.into_inner());
    hits.push(now());
}

fn clean(raw: &str, max: usize) -> String {
    let mut out = String::new();
    for ch in raw.chars().filter(|c| !c.is_control() || *c == '\n') {
        if ch == '\n' {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            continue;
        }
        if ch == ' ' && out.ends_with(' ') {
            continue;
        }
        out.push(ch);
        if out.chars().count() >= max {
            break;
        }
    }
    out.trim().to_string()
}

fn peak_label(report: &FitReport) -> String {
    let Some(p) = report.peaks.iter().max_by(|a, b| a.height.partial_cmp(&b.height).unwrap_or(std::cmp::Ordering::Equal)) else {
        return String::new();
    };
    let axis = match report.options.unit {
        lightmatter::measure::Unit::Cm => "cm⁻¹",
        lightmatter::measure::Unit::Ev | lightmatter::measure::Unit::Nm => "eV",
        lightmatter::measure::Unit::Other => "",
    };
    format!("{} {axis}", lightmatter::publish::num(p.position))
}

fn spark_of(report: &FitReport) -> Vec<f64> {
    let ys: &[f64] = if report.model_q.len() >= 8 {
        &report.model_q
    } else if report.data_y.len() >= 8 {
        &report.data_y
    } else if report.eps2.len() >= 8 {
        &report.eps2
    } else {
        return Vec::new();
    };
    let n = 48.min(ys.len());
    if n < 2 {
        return Vec::new();
    }
    (0..n)
        .map(|i| {
            let idx = i * (ys.len() - 1) / (n - 1);
            ys[idx]
        })
        .filter(|v| v.is_finite())
        .collect()
}

pub fn spark_svg(ys: &[f64]) -> String {
    if ys.len() < 2 {
        return String::new();
    }
    let lo = ys.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (hi - lo).max(1e-9);
    let pts = ys
        .iter()
        .enumerate()
        .map(|(i, y)| {
            let x = i as f64 / (ys.len() - 1) as f64 * 120.0;
            let yy = 32.0 - (*y - lo) / span * 28.0;
            format!("{x:.1},{yy:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    format!("<svg class=\"spark\" viewBox=\"0 0 120 36\" aria-hidden=\"true\"><polyline points=\"{pts}\" /></svg>")
}

fn redirect(loc: &str) -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, loc)
        .body(axum::body::Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
