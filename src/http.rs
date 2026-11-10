//! Endpoint del laboratorio y worker Resuma para ajustes persistentes.

use axum::body::Body;
use axum::extract::Multipart;
use axum::http::{header, Response, StatusCode};
use axum::response::IntoResponse;
use resuma::core::ResumaError;
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

pub fn fit_worker(
    input: serde_json::Value,
    context: resuma::exec::WorkerContext,
) -> resuma::exec::workers::WorkerFuture {
    Box::pin(async move {
        let id = input
            .get("job_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ResumaError::Validation("falta el identificador del trabajo".into()))?
            .to_string();
        if !store::valid_job_id(&id) {
            return Err(ResumaError::Validation("el identificador del trabajo no es válido".into()));
        }
        if let Some(status) = store::job_status(&id) {
            if status.state == store::JobState::Completed {
                return Ok(serde_json::json!({"report_id": status.report_id}));
            }
        }
        let _fit_slot = FIT_SLOTS
            .acquire()
            .await
            .map_err(|error| ResumaError::Other(error.to_string()))?;
        let job = store::load_fit_job(&id)
            .ok_or_else(|| ResumaError::Validation("no se pudo recuperar el espectro guardado".into()))?;
        store::mark_job_running(&id).map_err(ResumaError::Other)?;
        let source = job.source.clone();
        let fit = match context.run_blocking(move || fit_text(&job.text, &job.options)).await {
            Ok(fit) => fit,
            Err(error) => {
                crate::stats::record_error();
                let _ = store::mark_job_failed(&id, &error.to_string());
                return Err(error);
            }
        };
        let report = match fit {
            Ok(report) => report,
            Err(error) => {
                crate::stats::record_error();
                let _ = store::mark_job_failed(&id, &error);
                return Err(ResumaError::Other(error));
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
        let report_id = match store::save_report(&source, &report) {
            Ok(report_id) => report_id,
            Err(error) => {
                crate::stats::record_error();
                let _ = store::mark_job_failed(&id, &error);
                return Err(ResumaError::Other(error));
            }
        };
        store::mark_job_completed(&id, &report_id).map_err(ResumaError::Other)?;
        Ok(serde_json::json!({"report_id": report_id}))
    })
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
    let iter_max = iter.trim().parse::<usize>().unwrap_or(20).clamp(1, 800);
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
        if let Err(error) = resuma::exec::enqueue("lightmatter", "lightmatter_fit", serde_json::json!({"job_id": job_id})).await {
            crate::stats::record_error();
            let _ = store::mark_job_failed(&job_id, &error.to_string());
        }
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
