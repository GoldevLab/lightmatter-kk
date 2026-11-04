//! Idioma de esta petición: cookie `lm_lang`, y si no existe, Accept-Language.

use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use lightmatter::i18n::{self, Lang};

pub fn current() -> Lang {
    let req = resuma::prelude::current_request();
    let cookie = req.as_ref().and_then(|r| r.header("cookie"));
    let accept = req.as_ref().and_then(|r| r.header("accept-language"));
    i18n::resolve(cookie, accept)
}

/// Ruta actual, con la consulta, para volver después de elegir idioma.
pub fn here() -> String {
    let Some(req) = resuma::prelude::current_request() else {
        return "/".into();
    };
    if req.query.is_empty() {
        return req.path.clone();
    }
    let mut query = String::new();
    for (key, value) in &req.query {
        if !query.is_empty() {
            query.push('&');
        }
        query.push_str(&encode(key));
        query.push('=');
        query.push_str(&encode(value));
    }
    if query.is_empty() {
        req.path.clone()
    } else {
        format!("{}?{query}", req.path)
    }
}

#[derive(Deserialize)]
pub struct NextQuery {
    next: Option<String>,
}

pub async fn set_lang(Path(code): Path<String>, Query(q): Query<NextQuery>) -> Response<Body> {
    let Some(lang) = Lang::from_code(&code) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let next = safe_next(q.next.as_deref().unwrap_or("/"));
    let mut cookie = format!(
        "lm_lang={}; Path=/; Max-Age=31536000; HttpOnly; SameSite=Lax",
        lang.code()
    );
    if crate::stats::site_is_https() {
        cookie.push_str("; Secure");
    }
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, next)
        .header(header::SET_COOKIE, cookie)
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn safe_next(raw: &str) -> String {
    let raw = raw.trim();
    if raw.starts_with('/')
        && !raw.starts_with("//")
        && !raw.contains('\\')
        && !raw.contains('\n')
        && !raw.contains('\r')
    {
        raw.to_string()
    } else {
        "/".into()
    }
}

pub fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
