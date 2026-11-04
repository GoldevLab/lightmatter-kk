//! Sesión del panel. La clave no sale al navegador: solo una cookie firmada.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{header, Response, StatusCode};
use axum::response::IntoResponse;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::stats;

const COOKIE: &str = "lm_admin";
const SESSION_SECS: u64 = 12 * 60 * 60;
const USER: &str = "admin";
const PASSWORD: &str = "10203042";
const WINDOW_SECS: u64 = 10 * 60;
const MAX_FAILS: usize = 8;

type HmacSha256 = Hmac<Sha256>;

static FAILS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static SECRET: Mutex<Option<[u8; 32]>> = Mutex::new(None);

pub fn request_is_admin() -> bool {
    resuma::prelude::current_request()
        .map(|req| cookie_valid(req.header("cookie")))
        .unwrap_or(false)
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    user: String,
    password: String,
}

pub async fn login(axum::Form(form): axum::Form<LoginForm>) -> Response<Body> {
    if too_many_fails() {
        return redirect("/admin?e=2", None);
    }
    if !login_ok(&form.user, &form.password) {
        note_fail();
        return redirect("/admin?e=1", None);
    }
    clear_fails();
    redirect("/admin", Some(cookie_header(&issue(), SESSION_SECS as i64)))
}

pub async fn logout() -> Response<Body> {
    redirect("/admin", Some(cookie_header("", 0)))
}

fn login_ok(user: &str, password: &str) -> bool {
    ct_eq(user.trim(), USER) && ct_eq(password, PASSWORD)
}

fn too_many_fails() -> bool {
    let now = now_secs();
    let mut fails = FAILS.lock().unwrap_or_else(|e| e.into_inner());
    fails.retain(|t| now.saturating_sub(*t) < WINDOW_SECS);
    fails.len() >= MAX_FAILS
}

fn note_fail() {
    let mut fails = FAILS.lock().unwrap_or_else(|e| e.into_inner());
    fails.push(now_secs());
}

fn clear_fails() {
    FAILS.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

fn issue() -> String {
    let exp = now_secs().saturating_add(SESSION_SECS);
    let mac = hmac_hex(format!("lm-admin|{exp}").as_bytes());
    format!("{exp}.{mac}")
}

fn cookie_valid(header: Option<&str>) -> bool {
    let Some(raw) = header else {
        return false;
    };
    raw.split(';').any(|part| {
        part.trim()
            .strip_prefix("lm_admin=")
            .is_some_and(|value| session_valid(value.trim()))
    })
}

fn session_valid(raw: &str) -> bool {
    let Some((exp_s, mac)) = raw.split_once('.') else {
        return false;
    };
    if exp_s.is_empty() || mac.len() != 64 {
        return false;
    }
    let Ok(exp) = exp_s.parse::<u64>() else {
        return false;
    };
    let now = now_secs();
    if exp <= now || exp > now.saturating_add(SESSION_SECS).saturating_add(120) {
        return false;
    }
    hmac_ok(format!("lm-admin|{exp}").as_bytes(), mac)
}

fn cookie_header(value: &str, max_age: i64) -> String {
    let mut header = format!("{COOKIE}={value}; Path=/; Max-Age={max_age}; HttpOnly; SameSite=Lax");
    if stats::site_is_https() {
        header.push_str("; Secure");
    }
    header
}

fn redirect(loc: &str, cookie: Option<String>) -> Response<Body> {
    let mut builder = Response::builder().status(StatusCode::SEE_OTHER).header(header::LOCATION, loc);
    if let Some(cookie) = cookie {
        builder = builder.header(header::SET_COOKIE, cookie);
    }
    builder
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn secret() -> [u8; 32] {
    let mut slot = SECRET.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(bytes) = *slot {
        return bytes;
    }
    let bytes = load_or_create_secret();
    *slot = Some(bytes);
    bytes
}

fn load_or_create_secret() -> [u8; 32] {
    let path = data_dir().join("admin-secret");
    if let Ok(raw) = std::fs::read_to_string(&path) {
        if let Some(bytes) = decode_hex32(raw.trim()) {
            return bytes;
        }
    }
    let mut bytes = [0u8; 32];
    let _ = getrandom::getrandom(&mut bytes);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&path, hex_encode(&bytes));
    bytes
}

fn data_dir() -> PathBuf {
    if let Ok(root) = std::env::var("RESUMA_DATA_DIR") {
        if !root.trim().is_empty() {
            return PathBuf::from(root);
        }
    }
    PathBuf::from("data")
}

fn hmac_hex(data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(&secret()).expect("hmac key");
    mac.update(data);
    hex_encode(&mac.finalize().into_bytes())
}

fn hmac_ok(data: &[u8], sig_hex: &str) -> bool {
    let mut mac = HmacSha256::new_from_slice(&secret()).expect("hmac key");
    mac.update(data);
    let got = mac.finalize().into_bytes();
    let Some(want) = decode_hex(sig_hex) else {
        return false;
    };
    if got.len() != want.len() {
        return false;
    }
    got.iter().zip(want.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

fn ct_eq(left: &str, right: &str) -> bool {
    let a = left.as_bytes();
    let b = right.as_bytes();
    let mut diff = (a.len() ^ b.len()) as u8;
    let n = a.len().max(b.len()).max(1);
    for i in 0..n {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        diff |= x ^ y;
    }
    diff == 0
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

fn decode_hex32(raw: &str) -> Option<[u8; 32]> {
    let bytes = decode_hex(raw)?;
    if bytes.len() != 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Some(out)
}

fn decode_hex(raw: &str) -> Option<Vec<u8>> {
    if raw.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(raw.len() / 2);
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Some(out)
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
