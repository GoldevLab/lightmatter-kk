//! Idioma de esta petición: `Accept-Language`. Español si el navegador no coincide.

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

#[derive(Deserialize)]
pub struct NextQuery {
    next: Option<String>,
}

pub async fn set_lang(Path(code): Path<String>, Query(q): Query<NextQuery>) -> Response<Body> {
    let Some(_lang) = Lang::from_code(&code) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let next = safe_next(q.next.as_deref().unwrap_or("/"));
    let mut cookie = "lm_lang=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax".to_string();
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
