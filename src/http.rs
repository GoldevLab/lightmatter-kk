//! El ajuste corre en este proceso y la página siguiente lo pinta. Sin sesión.

use axum::body::Body;
use axum::extract::Multipart;
use axum::http::{header, Response, StatusCode};
use axum::response::IntoResponse;
use tokio::sync::Semaphore;

use lightmatter::fit::{export, fit_text, TextFit};

use crate::store;

const MAX_UPLOAD: usize = 4_000_000;
pub const MAX_REQUEST_BODY: usize = MAX_UPLOAD + 64_000;
static FIT_SLOTS: Semaphore = Semaphore::const_new(1);

fn redirect(loc: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, loc)
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn start_fit_job(id: String) {
    tokio::spawn(async move {
        let Ok(_fit_slot) = FIT_SLOTS.acquire().await else {
            let _ = store::mark_job_failed(&id, "el servidor cerró la cola de ajustes");
            return;
        };
        let Some(job) = store::load_fit_job(&id) else {
            let _ = store::mark_job_failed(&id, "no se pudo recuperar el espectro guardado");
            return;
        };
        if store::mark_job_running(&id).is_err() {
            return;
        }
        let source = job.source.clone();
        let fit = tokio::task::spawn_blocking(move || fit_text(&job.text, &job.options)).await;
        let report = match fit {
            Ok(Ok(report)) => report,
            Ok(Err(error)) => {
                crate::stats::record_error();
                let _ = store::mark_job_failed(&id, &error);
                return;
            }
            Err(_) => {
                crate::stats::record_error();
                let _ = store::mark_job_failed(&id, "el servidor no pudo completar el ajuste");
                return;
            }
        };
        crate::stats::record_fit(crate::stats::FitHit {
            at: 0,
            source: source.clone(),
            measure: report.measure().key().to_string(),
            unit: report.options.unit.label().to_string(),
            points: report.points_used,
            chisq: report.chisq,
            converged: report.converged,
        });
        match store::save_report(&source, &report) {
            Ok(report_id) => {
                if let Err(error) = store::mark_job_completed(&id, &report_id) {
                    let _ = store::mark_job_failed(&id, &error);
                }
            }
            Err(error) => {
                crate::stats::record_error();
                let _ = store::mark_job_failed(&id, &error);
            }
        }
    });
}

pub fn resume_fit_jobs() {
    for id in store::recoverable_jobs() {
        start_fit_job(id);
    }
}

pub async fn run_lab(mut form: Multipart) -> Response<Body> {
    let mut spectrum = String::new();
    let mut file_text = String::new();
    let mut file_name = String::new();
    let mut drude_on = false;
    let mut log_mesh = false;
    let mut extended = false;
    let mut eps_inf = String::new();
    let mut omega_p = String::new();
    let mut gamma = String::new();
    let mut iter = String::new();
    let mut smooth = String::new();
    let mut neg = String::new();
    let mut measure = String::new();
    let mut unit = String::new();
    let mut thickness = String::new();
    let mut temperature = String::new();
    let mut job_id = None;
    let mut bytes_seen = 0usize;

    while let Ok(Some(field)) = form.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        let fname = field.file_name().map(|s| s.to_string());
        let bytes = field.bytes().await.unwrap_or_default();
        bytes_seen = bytes_seen.saturating_add(bytes.len());
        if bytes_seen > MAX_UPLOAD {
            return redirect("/lab?e=El%20archivo%20supera%204%20MB");
        }
        if name == "file" {
            if let Some(n) = fname {
                if !n.is_empty() && !bytes.is_empty() {
                    file_name = safe_name(&n);
                    file_text = String::from_utf8_lossy(&bytes).into_owned();
                }
            }
            continue;
        }
        let value = String::from_utf8_lossy(&bytes).into_owned();
        match name.as_str() {
            "spectrum" => spectrum = value,
            "drude" => drude_on = value == "1" || value == "on",
            "log_mesh" => log_mesh = value == "1" || value == "on",
            "extended" => extended = value == "1" || value == "on",
            "eps_inf" => eps_inf = value,
            "omega_p" => omega_p = value,
            "gamma" => gamma = value,
            "iter" => iter = value,
            "smooth" => smooth = value,
            "neg" => neg = value,
            "measure" => measure = value,
            "unit" => unit = value,
            "thickness" => thickness = value,
            "temperature" => temperature = value,
            "job_id" => job_id = Some(value),
            _ => {}
        }
    }

    let source_name = if file_text.trim().is_empty() {
        "manual".into()
    } else {
        file_name
    };
    let text = if file_text.trim().is_empty() { spectrum } else { file_text };
    let iter_max = iter.trim().parse::<usize>().unwrap_or(20).clamp(1, 1000);
    let options = TextFit {
        iter_max,
        log_mesh,
        extended,
        drude_on,
        eps_inf,
        omega_p,
        gamma,
        smoothness: parse_nonneg(&smooth, 0.08),
        negativity: parse_nonneg(&neg, 100.0),
        measure,
        unit,
        thickness,
        temperature,
    };
    let (job_id, created) = match store::create_job(job_id.as_deref(), &source_name, &text, &options) {
        Ok(job) => job,
        Err(error) => return redirect(&format!("/lab?e={}", urlencode(&error))),
    };
    if created {
        start_fit_job(job_id.clone());
    }
    redirect(&format!("/lab?j={}", urlencode(&job_id)))
}

pub async fn job_status(axum::extract::Path(id): axum::extract::Path<String>) -> Response<Body> {
    if !store::valid_job_id(&id) {
        return StatusCode::NOT_FOUND.into_response();
    }
    match store::job_status(&id) {
        Some(status) => axum::Json(status).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn download(axum::extract::Path(id): axum::extract::Path<String>) -> Response<Body> {
    send(&id, "dat")
}

pub async fn download_as(axum::extract::Path((id, format)): axum::extract::Path<(String, String)>) -> Response<Body> {
    send(&id, &format)
}

fn send(id: &str, format: &str) -> Response<Body> {
    if !store::valid_id(id) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(loaded) = store::load_report(id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    deliver(id, &loaded.source, &loaded.report, format)
}

pub fn deliver(id: &str, source: &str, report: &lightmatter::fit::FitReport, format: &str) -> Response<Body> {
    let Some((body, mime, ext)) = (if format == "informe" {
        let lang = crate::locale::current();
        Some((
            lightmatter::informe::page(report, source, lang),
            "text/html; charset=utf-8",
            "html",
        ))
    } else {
        export(report, format, source)
    }) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    crate::stats::record_download(format);
    let filename = if format == "informe" {
        format!("lightmatter-{id}-informe.html")
    } else {
        format!("lightmatter-{id}.{ext}")
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{filename}\""))
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn parse_nonneg(raw: &str, fallback: f64) -> f64 {
    let raw = raw.trim();
    if raw.is_empty() {
        return fallback;
    }
    match raw.parse::<f64>() {
        Ok(v) if v.is_finite() && v >= 0.0 => v.min(1.0e6),
        _ => fallback,
    }
}

fn safe_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or("archivo");
    let clean: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .take(80)
        .collect();
    if clean.is_empty() { "archivo".into() } else { clean }
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
