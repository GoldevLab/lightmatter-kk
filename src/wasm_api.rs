//! La misma función de ajuste, compilada a WASM para la PWA sin red.

use wasm_bindgen::prelude::*;

use crate::chart::result_markup;
use crate::fit::{fit_csv, fit_text, peaks_csv, report_csv, report_dat, report_json, TextFit};
use crate::i18n::{tx, Lang};
use crate::informe;

#[wasm_bindgen]
pub fn adjust(text: &str, opts_json: &str) -> String {
    let opts = text_fit_from_json(opts_json);
    match fit_text(text, &opts) {
        Ok(report) => {
            let lang = opt_lang(opts_json);
            let here = tx(lang, "este aparato", "this device", "本机", "este aparelho");
            let html = result_markup(here, &report, "#", lang);
            serde_json::json!({
                "ok": true,
                "html": html,
                "files": {
                    "dat": report_dat(&report),
                    "csv": report_csv(&report),
                    "ajuste": fit_csv(&report),
                    "picos": if report.peaks.is_empty() { String::new() } else { peaks_csv(&report) },
                    "json": report_json(&report),
                    "informe": informe::page(&report, here, lang),
                }
            })
            .to_string()
        }
        Err(error) => serde_json::json!({ "ok": false, "error": error }).to_string(),
    }
}

fn opt_lang(raw: &str) -> Lang {
    let v = serde_json::from_str::<serde_json::Value>(raw).unwrap_or(serde_json::Value::Null);
    Lang::from_code(v.get("lang").and_then(|x| x.as_str()).unwrap_or("")).unwrap_or(Lang::Es)
}

fn text_fit_from_json(raw: &str) -> TextFit {
    let v = serde_json::from_str::<serde_json::Value>(raw).unwrap_or(serde_json::Value::Null);
    let flag = |key: &str| v.get(key).and_then(|x| x.as_bool()).unwrap_or(false);
    let text = |key: &str| {
        v.get(key)
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string()
    };
    let iter = v.get("iter").and_then(|x| x.as_u64()).unwrap_or(20) as usize;
    let num = |key: &str, fallback: f64| match v.get(key) {
        Some(serde_json::Value::Number(n)) => n.as_f64().filter(|x| x.is_finite() && *x >= 0.0).unwrap_or(fallback),
        Some(serde_json::Value::String(s)) => s.trim().parse::<f64>().ok().filter(|x| x.is_finite() && *x >= 0.0).unwrap_or(fallback),
        _ => fallback,
    };
    TextFit {
        iter_max: iter,
        log_mesh: flag("log_mesh"),
        extended: flag("extended"),
        drude_on: flag("drude"),
        eps_inf: text("eps_inf"),
        omega_p: text("omega_p"),
        gamma: text("gamma"),
        smoothness: num("smooth", 0.08).min(1.0e6),
        negativity: num("neg", 100.0).min(1.0e6),
        measure: text("measure"),
        unit: text("unit"),
        thickness: text("thickness"),
        temperature: text("temperature"),
    }
}
