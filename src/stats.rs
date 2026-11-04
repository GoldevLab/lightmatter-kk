//! Conteos del laboratorio. Viven en el disco del servidor, no en el navegador.
//! No se guarda el espectro, ni una IP, ni la clave del admin.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use lightmatter::i18n::{tx, Lang};

const MAX_RECENT: usize = 40;

static BOOK: Mutex<Option<Book>> = Mutex::new(None);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FitHit {
    pub at: u64,
    pub source: String,
    pub measure: String,
    pub unit: String,
    pub points: usize,
    pub chisq: f64,
    pub converged: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Book {
    #[serde(default)]
    home: u64,
    #[serde(default)]
    lab: u64,
    #[serde(default)]
    learn: u64,
    #[serde(default)]
    fits: u64,
    #[serde(default)]
    errors: u64,
    #[serde(default)]
    downloads: u64,
    #[serde(default)]
    measures: BTreeMap<String, u64>,
    #[serde(default)]
    formats: BTreeMap<String, u64>,
    #[serde(default)]
    recent: Vec<FitHit>,
}

impl Default for Book {
    fn default() -> Self {
        Self {
            home: 0,
            lab: 0,
            learn: 0,
            fits: 0,
            errors: 0,
            downloads: 0,
            measures: BTreeMap::new(),
            formats: BTreeMap::new(),
            recent: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Dash {
    pub home: u64,
    pub lab: u64,
    pub learn: u64,
    pub fits: u64,
    pub errors: u64,
    pub downloads: u64,
    pub measures: Vec<(String, u64)>,
    pub formats: Vec<(String, u64)>,
    pub recent: Vec<FitHit>,
}

pub fn site_is_https() -> bool {
    std::env::var("SITE_URL").map(|s| s.starts_with("https://")).unwrap_or(true)
}

pub fn record_visit(page: &str) {
    with_book(|book| {
        match page {
            "home" => book.home = book.home.saturating_add(1),
            "lab" => book.lab = book.lab.saturating_add(1),
            "learn" => book.learn = book.learn.saturating_add(1),
            _ => return,
        }
        book.save();
    });
}

pub fn record_fit(mut hit: FitHit) {
    hit.at = now_secs();
    with_book(|book| {
        book.fits = book.fits.saturating_add(1);
        let key = hit.measure.clone();
        *book.measures.entry(key).or_insert(0) += 1;
        book.recent.insert(0, hit);
        book.recent.truncate(MAX_RECENT);
        book.save();
    });
}

pub fn record_error() {
    with_book(|book| {
        book.errors = book.errors.saturating_add(1);
        book.save();
    });
}

pub fn record_download(format: &str) {
    with_book(|book| {
        book.downloads = book.downloads.saturating_add(1);
        let key = format.to_string();
        *book.formats.entry(key).or_insert(0) += 1;
        book.save();
    });
}

pub fn dashboard() -> Dash {
    with_book(|book| {
        let mut measures: Vec<(String, u64)> = book.measures.iter().map(|(k, v)| (k.clone(), *v)).collect();
        measures.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let mut formats: Vec<(String, u64)> = book.formats.iter().map(|(k, v)| (k.clone(), *v)).collect();
        formats.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        Dash {
            home: book.home,
            lab: book.lab,
            learn: book.learn,
            fits: book.fits,
            errors: book.errors,
            downloads: book.downloads,
            measures,
            formats,
            recent: book.recent.clone(),
        }
    })
}

fn with_book<T>(f: impl FnOnce(&mut Book) -> T) -> T {
    let mut slot = BOOK.lock().unwrap_or_else(|e| e.into_inner());
    if slot.is_none() {
        *slot = Some(Book::load());
    }
    f(slot.as_mut().expect("book"))
}

impl Book {
    fn load() -> Self {
        let Ok(raw) = std::fs::read_to_string(path()) else {
            return Self::default();
        };
        serde_json::from_str(&raw).unwrap_or_default()
    }

    fn save(&self) {
        let path = path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let Ok(body) = serde_json::to_string(self) else {
            return;
        };
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, body).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn path() -> PathBuf {
    data_dir().join("stats.json")
}

fn data_dir() -> PathBuf {
    if let Ok(root) = std::env::var("RESUMA_DATA_DIR") {
        if !root.trim().is_empty() {
            return PathBuf::from(root);
        }
    }
    PathBuf::from("data")
}

pub fn grouped(n: u64) -> String {
    let raw = n.to_string();
    let mut out = String::with_capacity(raw.len() + raw.len() / 3);
    for (i, ch) in raw.chars().enumerate() {
        if i > 0 && (raw.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

pub fn ago(secs: u64, lang: Lang) -> String {
    if secs == 0 {
        return "—".into();
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let age = now.saturating_sub(secs);
    if age < 60 {
        return tx(lang, "ahora", "now", "刚刚", "agora").into();
    }
    let (n, unit_es, unit_en, unit_zh, unit_pt) = if age < 3_600 {
        (age / 60, "min", "min", "分钟", "min")
    } else if age < 86_400 {
        (age / 3_600, "h", "h", "小时", "h")
    } else {
        (age / 86_400, "d", "d", "天", "d")
    };
    match lang {
        Lang::Es => format!("hace {n} {unit_es}"),
        Lang::En => format!("{n} {unit_en} ago"),
        Lang::Zh => format!("{n} {unit_zh}前"),
        Lang::Pt => format!("há {n} {unit_pt}"),
    }
}

pub fn measure_label(key: &str) -> String {
    match key {
        "eps2" => "ε₂",
        "eps1" => "ε₁",
        "n" => "n",
        "k" => "k",
        "r" => "R",
        "alpha" => "α",
        "t" => "T",
        "lum" => "Luminiscencia",
        other => other,
    }
    .to_string()
}

pub fn format_label(key: &str) -> String {
    match key {
        "dat" => ".dat",
        "csv" => "CSV",
        "ajuste" => "Datos y ajuste",
        "picos" => "Picos",
        "json" => "JSON",
        "informe" => "Informe",
        other => other,
    }
    .to_string()
}
