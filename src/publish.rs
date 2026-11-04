//! Nombre y texto sugeridos para publicar un ajuste en el estante.

use crate::fit::FitReport;
use crate::i18n::{tx, Lang};
use crate::measure::{Measure, Unit};

pub fn form(download_href: &str, report: &FitReport, lang: Lang) -> String {
    if download_href == "#" {
        return format!(
            "<section class=\"panel publish-card\"><p class=\"eyebrow\">{}</p><h3>{}</h3><p>{}</p></section>",
            tx(lang, "Estante", "Shelf", "书架", "Estante"),
            tx(lang, "Publicar este análisis", "Publish this analysis", "发布这次分析", "Publicar esta análise"),
            tx(
                lang,
                "Este ajuste se hizo en el aparato, sin red. Para dejarlo en el estante público, analízalo otra vez con conexión.",
                "This fit ran on the device, offline. To put it on the public shelf, run it again while online.",
                "这次拟合是在本机离线完成的。要放上公共书架，请联网再分析一次。",
                "Este ajuste foi feito no aparelho, sem rede. Para deixá-lo no estante público, analise de novo com conexão.",
            ),
        );
    }
    let Some(id) = report_id(download_href) else {
        return String::new();
    };
    let (title, summary) = suggest(report, lang);
    format!(
        "<section class=\"panel publish-card\"><p class=\"eyebrow\">{}</p><h3>{}</h3><p>{}</p><form method=\"post\" action=\"/shelf/publish\" class=\"lab-form\"><input type=\"hidden\" name=\"report_id\" value=\"{}\" /><label>{}<input name=\"title\" type=\"text\" maxlength=\"90\" required value=\"{}\" /></label><label>{}<textarea name=\"summary\" rows=\"4\" maxlength=\"700\" required>{}</textarea></label><button class=\"btn btn-primary\" type=\"submit\">{}</button></form></section>",
        tx(lang, "Estante", "Shelf", "书架", "Estante"),
        tx(lang, "Publicar este análisis", "Publish this analysis", "发布这次分析", "Publicar esta análise"),
        tx(
            lang,
            "El nombre y el texto salen de este resultado. Cámbialos si quieres. Al publicar, las gráficas quedan en el estante para que otras personas las vean. No hace falta una cuenta.",
            "The name and the text come from this result. Change them if you want. Publishing puts the charts on the shelf for other people to see. No account is required.",
            "名称和文字来自这次结果。你可以修改。发布后，图会留在书架上供别人查看。不需要账户。",
            "O nome e o texto saem deste resultado. Mude se quiser. Ao publicar, os gráficos ficam no estante para outras pessoas verem. Não precisa de conta.",
        ),
        esc(id),
        tx(lang, "Nombre", "Name", "名称", "Nome"),
        esc(&title),
        tx(lang, "Descripción", "Description", "说明", "Descrição"),
        esc(&summary),
        tx(lang, "Publicar en el estante", "Publish to the shelf", "发布到书架", "Publicar no estante"),
    )
}

pub fn suggest(report: &FitReport, lang: Lang) -> (String, String) {
    let measure = report.measure();
    let unit = report.options.unit;
    let kind = kind_name(measure, lang);
    let axis = axis_word(unit);
    let peak = report.peaks.iter().max_by(|a, b| a.height.partial_cmp(&b.height).unwrap_or(std::cmp::Ordering::Equal));
    let title = if measure == Measure::Lum {
        match lang {
            Lang::Es => format!("Luminiscencia, {} picos", report.peaks.len()),
            Lang::En => format!("Luminescence, {} peaks", report.peaks.len()),
            Lang::Zh => format!("发光，{} 个峰", report.peaks.len()),
            Lang::Pt => format!("Luminescência, {} picos", report.peaks.len()),
        }
    } else if let Some(p) = peak {
        match lang {
            Lang::Es => format!("{kind}, pico en {} {axis}", num(p.position)),
            Lang::En => format!("{kind}, peak at {} {axis}", num(p.position)),
            Lang::Zh => format!("{kind}，峰在 {} {axis}", num(p.position)),
            Lang::Pt => format!("{kind}, pico em {} {axis}", num(p.position)),
        }
    } else if let Some(g) = report.tauc_direct.or(report.tauc_indirect) {
        match lang {
            Lang::Es => format!("{kind}, borde cerca de {} eV", num(g)),
            Lang::En => format!("{kind}, edge near {} eV", num(g)),
            Lang::Zh => format!("{kind}，带边约 {} eV", num(g)),
            Lang::Pt => format!("{kind}, borda perto de {} eV", num(g)),
        }
    } else {
        match lang {
            Lang::Es => format!("{kind}, {} puntos", report.points_used),
            Lang::En => format!("{kind}, {} points", report.points_used),
            Lang::Zh => format!("{kind}，{} 个点", report.points_used),
            Lang::Pt => format!("{kind}, {} pontos", report.points_used),
        }
    };

    let (x0, x1) = span(&report.data_x);
    let mut summary = match lang {
        Lang::Es => format!(
            "Ajuste variacional de Kuzmenko de {kind}, {n} puntos entre {a} y {b} {axis}. χ² = {chi}.",
            n = report.points_used,
            a = num(x0),
            b = num(x1),
            chi = sci(report.chisq),
        ),
        Lang::En => format!(
            "Kuzmenko variational fit of {kind}, {n} points from {a} to {b} {axis}. χ² = {chi}.",
            n = report.points_used,
            a = num(x0),
            b = num(x1),
            chi = sci(report.chisq),
        ),
        Lang::Zh => format!(
            "{kind} 的 Kuzmenko 变分拟合，{n} 个点，从 {a} 到 {b} {axis}。χ² = {chi}。",
            n = report.points_used,
            a = num(x0),
            b = num(x1),
            chi = sci(report.chisq),
        ),
        Lang::Pt => format!(
            "Ajuste variacional de Kuzmenko de {kind}, {n} pontos entre {a} e {b} {axis}. χ² = {chi}.",
            n = report.points_used,
            a = num(x0),
            b = num(x1),
            chi = sci(report.chisq),
        ),
    };
    if let Some(v) = report.eps_inf {
        summary.push(' ');
        summary.push_str(&match lang {
            Lang::Es => format!("ε∞ = {}.", num(v)),
            Lang::En => format!("ε∞ = {}.", num(v)),
            Lang::Zh => format!("ε∞ = {}。", num(v)),
            Lang::Pt => format!("ε∞ = {}.", num(v)),
        });
    }
    if let Some(p) = peak {
        summary.push(' ');
        summary.push_str(&match lang {
            Lang::Es => format!("El pico mayor queda en {} {axis}.", num(p.position)),
            Lang::En => format!("The largest peak sits at {} {axis}.", num(p.position)),
            Lang::Zh => format!("最高峰在 {} {axis}。", num(p.position)),
            Lang::Pt => format!("O maior pico fica em {} {axis}.", num(p.position)),
        });
    }
    if let Some((lo, hi)) = finite_span(&report.n_index) {
        summary.push(' ');
        summary.push_str(&match lang {
            Lang::Es => format!("n va de {} a {}.", num(lo), num(hi)),
            Lang::En => format!("n runs from {} to {}.", num(lo), num(hi)),
            Lang::Zh => format!("n 从 {} 到 {}。", num(lo), num(hi)),
            Lang::Pt => format!("n vai de {} a {}.", num(lo), num(hi)),
        });
    }
    (squash(&title, 90), squash(&summary, 700))
}

fn kind_name(measure: Measure, lang: Lang) -> &'static str {
    match measure {
        Measure::Eps2 => "ε₂",
        Measure::Eps1 => "ε₁",
        Measure::N => "n",
        Measure::K => "k",
        Measure::R => tx(lang, "Reflectividad", "Reflectivity", "反射率", "Refletividade"),
        Measure::Alpha => "α",
        Measure::T => tx(lang, "Transmitancia", "Transmittance", "透射率", "Transmitância"),
        Measure::Lum => tx(lang, "Luminiscencia", "Luminescence", "发光", "Luminescência"),
    }
}

fn axis_word(unit: Unit) -> &'static str {
    match unit {
        Unit::Cm => "cm⁻¹",
        Unit::Ev | Unit::Nm => "eV",
        Unit::Other => "",
    }
}

fn span(xs: &[f64]) -> (f64, f64) {
    let a = xs.first().copied().unwrap_or(0.0);
    let b = xs.last().copied().unwrap_or(a);
    (a, b)
}

fn finite_span(xs: &[f64]) -> Option<(f64, f64)> {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for v in xs.iter().copied().filter(|v| v.is_finite()) {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if lo.is_finite() && hi.is_finite() { Some((lo, hi)) } else { None }
}

pub fn num(v: f64) -> String {
    if !v.is_finite() {
        return "—".into();
    }
    let a = v.abs();
    if a != 0.0 && (a < 0.01 || a >= 10_000.0) {
        format!("{v:.3e}")
    } else if a >= 100.0 {
        format!("{v:.1}")
    } else if a >= 10.0 {
        format!("{v:.2}")
    } else {
        format!("{v:.3}")
    }
}

fn sci(v: f64) -> String {
    if !v.is_finite() { "—".into() } else { format!("{v:.3e}") }
}

fn squash(raw: &str, max: usize) -> String {
    let mut out = String::new();
    for ch in raw.chars().filter(|c| !c.is_control()) {
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

fn report_id(href: &str) -> Option<&str> {
    let rest = href.strip_prefix("/lab/download/")?;
    let id = rest.split('/').next().unwrap_or("");
    let body = id.strip_prefix("fit_")?;
    if body.len() <= 40 && body.contains('_') && body.chars().all(|c| c.is_ascii_hexdigit() || c == '_') {
        Some(id)
    } else {
        None
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
