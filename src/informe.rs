//! Informe HTML de un ajuste: un solo archivo, con las tablas del resultado.

use crate::fit::FitReport;
use crate::i18n::{tx, Lang};
use crate::measure::{Measure, Peak, Unit, HC_EV_NM};

pub fn html(report: &FitReport, source: &str) -> String {
    page(report, source, Lang::Es)
}

pub fn page(report: &FitReport, source: &str, lang: Lang) -> String {
    let measure = report.measure();
    let unit = report.options.unit;
    let mut out = String::with_capacity(64 * 1024);
    let title = title(measure, lang);
    out.push_str(&head(&title, lang));
    out.push_str("<main>");
    out.push_str(&format!(
        "<p class=\"kicker\">Lightmatter · {}</p>",
        tx(lang, "Kramers–Kronig variacional", "variational Kramers–Kronig", "变分克拉默斯–克罗尼格", "Kramers–Kronig variacional")
    ));
    out.push_str("<h1>");
    out.push_str(&esc(&title));
    out.push_str("</h1>");
    out.push_str("<p class=\"lead\">");
    out.push_str(tx(
        lang,
        "Restricción variacional de Kuzmenko, Review of Scientific Instruments 76, 083108 (2005). ",
        "Kuzmenko’s variational constraint, Review of Scientific Instruments 76, 083108 (2005). ",
        "Kuzmenko 的变分约束，Review of Scientific Instruments 76, 083108 (2005)。",
        "Restrição variacional de Kuzmenko, Review of Scientific Instruments 76, 083108 (2005). ",
    ));
    out.push_str(tx(
        lang,
        "ε₂ es una suma de triángulos; ε₁ es ε∞ más la transformada discreta de Kramers–Kronig de esos triángulos. ",
        "ε₂ is a sum of triangles; ε₁ is ε∞ plus the discrete Kramers–Kronig transform of those triangles. ",
        "ε₂ 是三角基之和；ε₁ 是 ε∞ 加上这些三角的离散克拉默斯–克罗尼格变换。",
        "ε₂ é uma soma de triângulos; ε₁ é ε∞ mais a transformada discreta de Kramers–Kronig desses triângulos. ",
    ));
    if measure == Measure::Lum {
        out.push_str(tx(lang, "Esta medida es emisión: no pasa por Kramers–Kronig.", "This measurement is emission: it does not go through Kramers–Kronig.", "这项测量是发射：不经过克拉默斯–克罗尼格。", "Esta medida é emissão: não passa por Kramers–Kronig."));
    } else {
        out.push_str(tx(lang, "El ajuste compara la magnitud medida, no ε₂, salvo que se haya medido ε₂.", "The fit compares the measured quantity, not ε₂, unless ε₂ was what you measured.", "拟合比较的是测量量，不是 ε₂，除非测量的就是 ε₂。", "O ajuste compara a grandeza medida, não ε₂, salvo que se tenha medido ε₂."));
    }
    out.push_str("</p>");
    out.push_str("<p class=\"meta\">");
    out.push_str(&esc(source.trim()));
    if !source.trim().is_empty() {
        out.push_str(" · ");
    }
    out.push_str(&esc(&format!(
        "{} · {}",
        crate::i18n::measure_name(lang, measure.key()),
        crate::i18n::unit_name(lang, unit.key())
    )));
    out.push_str("</p>");
    out.push_str("<p class=\"note\">");
    out.push_str(axis_note(unit, lang));
    out.push_str("</p>");

    let mut n = 0u32;
    conditions(&mut out, report, &mut n, lang);
    highlights(&mut out, report, &mut n, lang);
    warnings(&mut out, report, &mut n, lang);
    peaks(&mut out, report, &mut n, lang);
    if measure == Measure::Lum {
        luminescence_spectrum(&mut out, report, &mut n, lang);
    } else {
        comparison(&mut out, report, &mut n, lang);
        optics(&mut out, report, &mut n, lang);
        extended(&mut out, report, &mut n, lang);
    }
    definitions(&mut out, report, &mut n, lang);
    out.push_str(&format!(
        "<footer><p>{}</p></footer>",
        tx(
            lang,
            "Lightmatter. El cálculo es el del laboratorio: Rust, sin cuenta. Este archivo no es la empresa de computación fotónica del mismo nombre.",
            "Lightmatter. The calculation is the laboratory’s: Rust, no account. This file is not the photonic-computing company of the same name.",
            "Lightmatter。计算就是实验室里的：Rust，无需账户。本文件不是同名的光子计算公司。",
            "Lightmatter. O cálculo é o do laboratório: Rust, sem conta. Este arquivo não é a empresa de computação fotônica do mesmo nome.",
        )
    ));
    out.push_str("</main></body></html>");
    out
}

fn title(measure: Measure, lang: Lang) -> String {
    if measure == Measure::Lum {
        tx(lang, "Informe de luminiscencia", "Luminescence report", "发光报告", "Relatório de luminescência").into()
    } else {
        format!(
            "{} {}",
            tx(lang, "Informe del ajuste de", "Fit report for", "拟合报告：", "Relatório do ajuste de"),
            measure.symbol()
        )
    }
}

fn head(title: &str, lang: Lang) -> String {
    let lang = lang.html();
    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{title}</title>
<style>
:root {{ color-scheme: light; }}
* {{ box-sizing: border-box; }}
body {{ margin: 0; background: #f3efe6; color: #1c1915; font: 16px/1.55 "Palatino Linotype", Palatino, "Times New Roman", serif; }}
main {{ max-width: 1080px; margin: 0 auto; padding: 2.4rem 1.4rem 4rem; background: #fff; }}
.kicker {{ margin: 0 0 0.35rem; font: 600 0.72rem/1.4 "Segoe UI", sans-serif; letter-spacing: 0.16em; text-transform: uppercase; color: #0c6b5c; }}
h1 {{ margin: 0 0 0.6rem; font-size: 1.85rem; font-weight: 600; letter-spacing: -0.02em; }}
h2 {{ margin: 2rem 0 0.55rem; font-size: 1.2rem; font-weight: 600; border-bottom: 1px solid #1c1915; padding-bottom: 0.2rem; }}
.lead, .note, .meta, footer p {{ color: #4e493f; }}
.meta {{ font: 0.92rem/1.45 "Segoe UI", sans-serif; }}
table {{ width: 100%; border-collapse: collapse; margin: 0.2rem 0 0.4rem; font: 0.86rem/1.35 "Segoe UI", sans-serif; font-variant-numeric: tabular-nums; }}
caption {{ caption-side: bottom; text-align: left; color: #5c564c; font: italic 0.84rem/1.4 "Palatino Linotype", Palatino, serif; padding: 0.45rem 0 0; }}
th, td {{ border-bottom: 1px solid #e6e0d4; padding: 0.38rem 0.55rem; text-align: right; vertical-align: top; }}
th.text, td.text {{ text-align: left; }}
thead th {{ border-bottom: 2px solid #1c1915; font-size: 0.72rem; letter-spacing: 0.04em; text-transform: uppercase; background: #fff; position: sticky; top: 0; }}
tbody tr:nth-child(even) {{ background: #faf8f3; }}
tr.key td {{ background: #e5f5f0; font-weight: 650; }}
tr.key td.text {{ box-shadow: inset 3px 0 0 #0c6b5c; }}
ul {{ margin: 0.2rem 0 0; padding-left: 1.15rem; }}
li {{ margin: 0.28rem 0; }}
footer {{ margin-top: 2.4rem; border-top: 1px solid #e6e0d4; padding-top: 0.8rem; font-size: 0.88rem; }}
code {{ font-family: "Cascadia Mono", Consolas, monospace; font-size: 0.92em; }}
@media print {{
  body {{ background: #fff; }}
  main {{ max-width: none; padding: 0; }}
  thead th {{ position: static; }}
  h2, tr.key {{ break-after: avoid; }}
  tr {{ break-inside: avoid; }}
}}
</style>
</head>
<body>
"#
    )
}

fn h2(out: &mut String, n: &mut u32, title: &str) {
    *n += 1;
    out.push_str("<h2>");
    out.push_str(&n.to_string());
    out.push_str(" · ");
    out.push_str(title);
    out.push_str("</h2>");
}

fn conditions(out: &mut String, report: &FitReport, n: &mut u32, lang: Lang) {
    let measure = report.measure();
    let o = &report.options;
    h2(out, n, tx(lang, "Condiciones del cálculo", "Conditions of the calculation", "计算条件", "Condições do cálculo"));
    out.push_str("<table><thead><tr><th class=\"text\">Condición</th><th>Valor</th></tr></thead><tbody>");
    kv(out, "Medida", measure.label(), false);
    kv(out, "Unidad del archivo", o.unit.label(), false);
    kv(out, "Eje de las tablas", o.unit.axis(), false);
    kv(out, "Puntos en el ajuste", &report.points_used.to_string(), false);
    if measure != Measure::Lum {
        kv(out, "Anclas de ε₂", &report.n_anchor.to_string(), false);
        kv(out, "Coeficientes libres", &report.mfit.to_string(), false);
        kv(out, "Pasos", &status_line(report), false);
        kv(out, "Suavidad", &fmt(o.smoothness), false);
        kv(out, "Castigo si ε₂ < 0", &fmt(o.negativity), false);
        kv(out, "Eje en logaritmo", if o.log_mesh { "forzado" } else { "solo si el rango supera ×20" }, false);
        kv(out, "Cola fuera del rango", if o.extended { "dibujada" } else { "entra en ε₁ y no se tabula al final" }, false);
        if o.thickness_um > 0.0 {
            kv(out, "Espesor de la lámina", &format!("{} µm", fmt(o.thickness_um)), false);
        }
        match o.eps_inf {
            Some(v) => kv(out, "ε∞ escrito", &fmt(v), false),
            None => kv(out, "ε∞ escrito", "vacío", false),
        }
        if let Some(d) = &report.drude {
            kv(out, "Drude ωₚ", &fmt(d.omega_p), false);
            kv(out, "Drude γ", &fmt(d.gamma), false);
        } else {
            kv(out, "Fondo Drude", "no", false);
        }
    } else {
        kv(out, "Temperatura de la muestra", &format!("{} K", fmt(o.temperature_k)), false);
    }
    out.push_str("</tbody></table>");
}

fn highlights(out: &mut String, report: &FitReport, n: &mut u32, lang: Lang) {
    let measure = report.measure();
    h2(out, n, tx(lang, "Resultados principales", "Main results", "主要结果", "Resultados principais"));
    out.push_str("<table><thead><tr><th class=\"text\">Resultado</th><th>Valor</th></tr></thead><tbody>");
    if measure == Measure::Lum {
        kv(out, "Picos", &report.peaks.len().to_string(), true);
        if let Some(c) = report.centroid {
            let extra = lambda_note(report.options.unit, c);
            kv(out, "Centroide", &format!("{}{extra}", fmt(c)), true);
        }
        if let Some(i) = report.integral {
            kv(out, "Intensidad integrada", &fmt(i), true);
        }
        if let Some(t) = report.carrier_temp {
            kv(out, "Temperatura de la cola alta", &format!("{} K", fmt(t)), true);
        }
    } else {
        kv(out, "χ²", &fmt(report.chisq), true);
        let dof = report.points_used.saturating_sub(report.mfit);
        if dof == 0 {
            kv(out, "χ²/dof", "no aplica: hay un coeficiente por punto", false);
        } else {
            kv(out, "χ²/dof", &fmt(report.chisq_dof), true);
        }
        if let Some(v) = report.eps_inf {
            let err = report.eps_inf_err.map(|e| format!(" ± {}", fmt(e))).unwrap_or_default();
            let note = if report.eps_inf_note.is_empty() {
                String::new()
            } else {
                format!(" · {}", report.eps_inf_note)
            };
            kv(out, "ε∞", &format!("{}{err}{note}", fmt(v)), true);
        }
        if let Some((rms, worst, at)) = residual(report) {
            kv(
                out,
                "RMS del residuo",
                &format!("{}  (raíz del promedio de (ajuste − dato)², sin el error)", fmt(rms)),
                false,
            );
            kv(
                out,
                "Punto más alejado",
                &format!("|ajuste − dato| = {} en {}", fmt(worst), fmt(at)),
                true,
            );
        }
        if let Some(p) = tallest(&report.peaks) {
            kv(
                out,
                "Pico mayor de ε₂",
                &format!("{} · altura {} · FWHM {}", fmt(p.position), fmt(p.height), fwhm_text(p.fwhm)),
                true,
            );
        }
        if let Some(g) = report.tauc_direct {
            let r2 = report.tauc_direct_r2.map(|v| format!(" · R² = {}", fmt(v))).unwrap_or_default();
            kv(out, "Corte de Tauc directo", &format!("{} eV{r2}", fmt(g)), true);
        }
        if let Some(g) = report.tauc_indirect {
            let r2 = report.tauc_indirect_r2.map(|v| format!(" · R² = {}", fmt(v))).unwrap_or_default();
            kv(out, "Corte de Tauc indirecto", &format!("{} eV{r2}", fmt(g)), true);
        }
        if !report.tauc_note.is_empty() {
            kv(out, "Lectura del borde", &report.tauc_note, false);
        }
    }
    out.push_str("</tbody></table>");
    out.push_str("<p class=\"note\">Las filas en verde son las que resumen el ajuste. El resto del archivo es el detalle, punto a punto.</p>");
}

fn warnings(out: &mut String, report: &FitReport, n: &mut u32, lang: Lang) {
    h2(out, n, tx(lang, "Avisos", "Warnings", "提示", "Avisos"));
    if report.warnings.is_empty() {
        out.push_str("<p>Ninguno.</p>");
        return;
    }
    out.push_str("<ul>");
    for w in &report.warnings {
        out.push_str("<li>");
        out.push_str(&esc(w));
        out.push_str("</li>");
    }
    out.push_str("</ul>");
}

fn peaks(out: &mut String, report: &FitReport, n: &mut u32, lang: Lang) {
    let what = if report.measure() == Measure::Lum { "la intensidad" } else { "ε₂" };
    h2(out, n, tx(lang, "Picos", "Peaks", "峰", "Picos"));
    if report.peaks.is_empty() {
        out.push_str("<p>Ningún máximo local pasa del 5 % de prominencia.</p>");
        return;
    }
    let unit = report.options.unit;
    let energy = unit.known();
    let total: f64 = report.peaks.iter().map(|p| p.area).sum();
    let best = tallest(&report.peaks).map(|p| p.position);
    out.push_str("<table><thead><tr><th class=\"text\">#</th><th>");
    out.push_str(&esc(unit.axis()));
    out.push_str("</th>");
    if energy {
        out.push_str("<th>λ (nm)</th>");
    }
    out.push_str("<th>FWHM</th><th>Altura</th><th>Prominencia</th><th>Área</th><th>% del área</th></tr></thead><tbody>");
    for (i, p) in report.peaks.iter().enumerate() {
        let key = best.is_some_and(|b| (b - p.position).abs() <= 1e-9);
        out.push_str(if key { "<tr class=\"key\">" } else { "<tr>" });
        cell_text(out, &format!("{}", i + 1));
        cell(out, &fmt(p.position));
        if energy {
            let lam = unit.energy(p.position).filter(|e| *e > 0.0).map(|e| fmt(HC_EV_NM / e)).unwrap_or_else(|| "—".into());
            cell(out, &lam);
        }
        cell(out, &fwhm_text(p.fwhm));
        cell(out, &fmt(p.height));
        cell(out, &fmt(p.prominence));
        cell(out, &fmt(p.area));
        let share = if total > 0.0 { format!("{} %", fmt(100.0 * p.area / total)) } else { "—".into() };
        cell(out, &share);
        out.push_str("</tr>");
    }
    out.push_str("</tbody><caption>Máximos locales de ");
    out.push_str(what);
    out.push_str(" cuya prominencia pasa del 5 % del rango. La fila verde es el más alto. FWHM es el ancho a media altura sobre la base local; «abierto» si esa mitad no se alcanza dentro del rango. El área es la integral del pico sobre su base.</caption></table>");
}

fn comparison(out: &mut String, report: &FitReport, section: &mut u32, lang: Lang) {
    let n = report.data_x.len();
    let sym = report.measure().symbol();
    let model = model_of(report);
    let with_sigma = report.data_sigma.len() == n && report.errors_given;
    let with_err = report.q_err.len() == n;
    h2(out, section, tx(lang, "Datos y ajuste", "Data and fit", "数据与拟合", "Dados e ajuste"));
    out.push_str("<table><thead><tr><th class=\"text\">");
    out.push_str(&esc(report.options.unit.axis()));
    out.push_str("</th><th>");
    out.push_str(&esc(&format!("{sym} dato")));
    out.push_str("</th>");
    if with_sigma {
        out.push_str("<th>error</th>");
    }
    if model.is_some() {
        out.push_str("<th>");
        out.push_str(&esc(&format!("{sym} ajuste")));
        out.push_str("</th><th>ajuste − dato</th>");
    }
    if with_err {
        out.push_str("<th>±1σ</th>");
    }
    out.push_str("</tr></thead><tbody>");
    for i in 0..n {
        out.push_str("<tr>");
        cell_text(out, &fmt(report.data_x[i]));
        cell(out, &fmt(report.data_y[i]));
        if with_sigma {
            cell(out, &fmt(report.data_sigma[i]));
        }
        if let Some(m) = model {
            cell(out, &fmt(m[i]));
            cell(out, &fmt(m[i] - report.data_y[i]));
        }
        if with_err {
            cell(out, &fmt(report.q_err[i]));
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody><caption>Cada fila es un punto medido. La columna del ajuste es la magnitud pedida, calculada desde el par (ε₁, ε₂). La diferencia es ajuste menos dato, en las mismas unidades. ±1σ sale de la covarianza de los datos y solo aparece si el archivo traía la tercera columna.</caption></table>");
}

fn optics(out: &mut String, report: &FitReport, section: &mut u32, lang: Lang) {
    let n = report.omega.len();
    if n == 0 || report.eps1.len() != n || report.eps2.len() != n {
        return;
    }
    let with_nk = report.n_index.len() == n && report.k_index.len() == n && report.reflectivity.len() == n;
    let with_loss = report.loss.len() == n;
    let with_err = report.coef_err.len() == n;
    let with_units = report.sigma1.len() == n && report.sigma2.len() == n && report.absorption.len() == n;
    let with_film = report.transmittance.len() == n && report.absorbance.len() == n;
    h2(out, section, tx(lang, "Constantes ópticas", "Optical constants", "光学常数", "Constantes ópticas"));
    out.push_str("<table><thead><tr><th class=\"text\">");
    out.push_str(&esc(report.options.unit.axis()));
    out.push_str("</th><th>ε₁</th><th>ε₂</th>");
    if with_err {
        out.push_str("<th>±1σ(ε₂)</th>");
    }
    if with_nk {
        out.push_str("<th>n</th><th>k</th><th>R</th>");
    }
    if with_loss {
        out.push_str("<th>−Im(1/ε)</th>");
    }
    if with_units {
        out.push_str("<th>σ₁</th><th>σ₂</th><th>α</th>");
    }
    if with_film {
        out.push_str("<th>T</th><th>A</th>");
    }
    out.push_str("</tr></thead><tbody>");
    for i in 0..n {
        out.push_str("<tr>");
        cell_text(out, &fmt(report.omega[i]));
        cell(out, &fmt(report.eps1[i]));
        cell(out, &fmt(report.eps2[i]));
        if with_err {
            cell(out, &fmt(report.coef_err[i]));
        }
        if with_nk {
            cell(out, &fmt(report.n_index[i]));
            cell(out, &fmt(report.k_index[i]));
            cell(out, &fmt(report.reflectivity[i]));
        }
        if with_loss {
            cell(out, &fmt(report.loss[i]));
        }
        if with_units {
            cell(out, &fmt(report.sigma1[i]));
            cell(out, &fmt(report.sigma2[i]));
            cell(out, &fmt(report.absorption[i]));
        }
        if with_film {
            cell(out, &fmt(report.transmittance[i]));
            cell(out, &fmt(report.absorbance[i]));
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody><caption>Valores en las anclas del modelo. σ₁ y σ₂ están en Ω⁻¹ cm⁻¹, α en cm⁻¹. T es la transmitancia de una lámina en aire y A = −log₁₀(T); salen cuando hay espesor y unidad de ω. R es la reflectividad de un medio semi-infinito a incidencia normal.</caption></table>");
}

fn extended(out: &mut String, report: &FitReport, section: &mut u32, lang: Lang) {
    let n = report.ext_omega.len();
    if n == 0 || report.ext_eps1.len() != n || report.ext_eps2.len() != n {
        return;
    }
    h2(out, section, tx(lang, "Fuera del rango medido", "Outside the measured range", "测量范围之外", "Fora da faixa medida"));
    out.push_str("<table><thead><tr><th class=\"text\">");
    out.push_str(&esc(report.options.unit.axis()));
    out.push_str("</th><th>ε₁</th><th>ε₂</th></tr></thead><tbody>");
    for i in 0..n {
        out.push_str("<tr>");
        cell_text(out, &fmt(report.ext_omega[i]));
        cell(out, &fmt(report.ext_eps1[i]));
        cell(out, &fmt(report.ext_eps2[i]));
        out.push_str("</tr>");
    }
    out.push_str("</tbody><caption>Extrapolación. ε₂ baja en línea recta hasta cero fuera de la ventana medida, y esa cola ya entró en ε₁. No son datos nuevos.</caption></table>");
}

fn luminescence_spectrum(out: &mut String, report: &FitReport, section: &mut u32, lang: Lang) {
    let n = report.data_x.len();
    let with_vrs = report.vrs_y.len() == n;
    h2(out, section, tx(lang, "Espectro", "Spectrum", "光谱", "Espectro"));
    out.push_str("<table><thead><tr><th class=\"text\">");
    out.push_str(&esc(report.options.unit.axis()));
    out.push_str("</th><th>I</th>");
    if with_vrs {
        out.push_str("<th>α / α máx</th>");
    }
    out.push_str("</tr></thead><tbody>");
    for i in 0..n {
        out.push_str("<tr>");
        cell_text(out, &fmt(report.data_x[i]));
        cell(out, &fmt(report.data_y[i]));
        if with_vrs {
            cell(out, &fmt(report.vrs_y[i]));
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody><caption>Intensidad medida");
    if with_vrs {
        out.push_str(". α relativa es la forma de van Roosbroeck–Shockley, α ∝ I · exp(E/kT) / E², normalizada a 1, con la temperatura de la muestra");
    }
    out.push_str(".</caption></table>");
}

fn definitions(out: &mut String, report: &FitReport, section: &mut u32, lang: Lang) {
    h2(out, section, tx(lang, "Definiciones", "Definitions", "定义", "Definições"));
    out.push_str("<ul>");
    if report.measure() == Measure::Lum {
        item(out, "Pico. Máximo local cuya prominencia —altura sobre el valle más alto a los lados— pasa del 5 % del rango. Se listan hasta 12.");
        item(out, "Centroide. Σ x·I / Σ I del espectro entero. La integral es por trapecios.");
        item(out, "Temperatura de la cola alta. En el tramo que baja del 50 % al 5 % del máximo se ajusta ln(I/E²). Sale solo con el eje en energía, si esa recta tiene R² de al menos 0,97 y la temperatura queda por debajo de 5000 K.");
        item(out, "van Roosbroeck–Shockley. En equilibrio, la forma de la absorción es proporcional a I·exp(E/kT)/E². Usa la temperatura de la muestra, no la de la cola.");
    } else {
        item(out, "χ². Suma de ((dato − modelo) / error)² de la magnitud medida. Sin tercera columna el error vale 1. La suavidad y el castigo de ε₂ negativa no entran en este número.");
        item(out, "ε∞. Parte real por encima de la ventana, la contribución de lo que no se midió a más alta frecuencia.");
        item(out, "ε₂. Suma de triángulos de altura Aₖ en anclas fijas, más el ε₂ de Drude si se pidió. A los lados de la ventana baja en línea recta hasta cero.");
        item(out, "ε₁. ε∞ más la Kramers–Kronig de ese ε₂. El kernel es g(x, y) = (x+y) ln|x+y| − (x−y) ln|x−y|, y el término central se resta.");
        item(out, "n y k. n = √((|ε| + ε₁) / 2) y k = signo(ε₂) √((|ε| − ε₁) / 2), con |ε| = √(ε₁² + ε₂²). Cumplen ε₁ = n² − k² y ε₂ = 2nk.");
        item(out, "R. ((n−1)² + k²) / ((n+1)² + k²). Reflectividad de un semiespacio a incidencia normal, no la de una película.");
        item(out, "−Im(1/ε). ε₂ / (ε₁² + ε₂²). Sus picos marcan modos longitudinales y el plasmón.");
        if report.options.unit.known() {
            item(out, "α = 4π ν̃ k, en cm⁻¹. σ₁ = ν̃ ε₂ / 60 y σ₂ = ν̃ (ε∞ − ε₁) / 60, en Ω⁻¹ cm⁻¹.");
            item(out, "T y A. Con espesor d en cm y x = exp(−α d), T = (1−R)² x / (1 − R² x²) para una lámina en aire, sumando reflexiones múltiples incoherentes. A = −log₁₀(T).");
            item(out, "Tauc. Corte de la tangente más empinada de (αE)², borde directo, o de (αE)^½, borde indirecto, antes del máximo.");
        }
    }
    out.push_str("</ul>");
}

fn status_line(report: &FitReport) -> String {
    let budget = report.options.iter_max.clamp(1, 500);
    let steps = if report.converged {
        report.iters_used.saturating_sub(1).max(1).min(budget)
    } else {
        report.iters_used.min(budget)
    };
    if report.iters_used == 0 {
        return format!("sin pasos registrados · tope {budget}");
    }
    if report.converged {
        format!("{steps} de {budget} · paró: χ² ya no bajaba un 1 % y ningún punto queda a más del 2 % del rango")
    } else if steps >= budget {
        format!("{steps} de {budget} · llegó al tope y algún punto sigue lejos. El espectro es el mejor aceptado")
    } else {
        format!("{steps} de {budget} · paró antes del tope. El espectro es el mejor aceptado")
    }
}

fn residual(report: &FitReport) -> Option<(f64, f64, f64)> {
    let n = report.data_x.len();
    let model = model_of(report)?;
    let mut ss = 0.0;
    let mut count = 0.0;
    let mut worst = 0.0;
    let mut at = report.data_x.first().copied().unwrap_or(0.0);
    for i in 0..n {
        let d = model[i] - report.data_y[i];
        if !d.is_finite() {
            continue;
        }
        ss += d * d;
        count += 1.0;
        if d.abs() >= worst {
            worst = d.abs();
            at = report.data_x[i];
        }
    }
    if count == 0.0 {
        return None;
    }
    Some(((ss / count).sqrt(), worst, at))
}

fn model_of(report: &FitReport) -> Option<&[f64]> {
    let n = report.data_x.len();
    if report.model_q.len() == n {
        Some(&report.model_q)
    } else if report.model_eps2.len() == n {
        Some(&report.model_eps2)
    } else {
        None
    }
}

fn tallest(peaks: &[Peak]) -> Option<&Peak> {
    peaks.iter().max_by(|a, b| a.height.partial_cmp(&b.height).unwrap_or(std::cmp::Ordering::Equal))
}

fn fwhm_text(fwhm: Option<f64>) -> String {
    fwhm.map(fmt).unwrap_or_else(|| "abierto".into())
}

fn lambda_note(unit: Unit, x: f64) -> String {
    unit.energy(x)
        .filter(|e| *e > 0.0)
        .map(|e| format!(" · {} nm", fmt(HC_EV_NM / e)))
        .unwrap_or_default()
}

fn axis_note(unit: Unit, lang: Lang) -> &'static str {
    match unit {
        Unit::Nm => tx(lang, "El archivo estaba en nm. Las tablas usan E = 1239,84193 / λ, en eV.", "The file was in nm. The tables use E = 1239.84193 / λ, in eV.", "文件原来是 nm。表格使用 E = 1239.84193 / λ，单位 eV。", "O arquivo estava em nm. As tabelas usam E = 1239,84193 / λ, em eV."),
        Unit::Cm => tx(lang, "El eje es el número de onda, en cm⁻¹.", "The axis is the wavenumber, in cm⁻¹.", "坐标轴是波数，单位 cm⁻¹。", "O eixo é o número de onda, em cm⁻¹."),
        Unit::Ev => tx(lang, "El eje es la energía, en eV.", "The axis is the energy, in eV.", "坐标轴是能量，单位 eV。", "O eixo é a energia, em eV."),
        Unit::Other => tx(lang, "El eje no tiene unidad física. No hay α, conductividad ni brecha de Tauc.", "The axis has no physical unit. There is no α, conductivity, or Tauc gap.", "坐标轴没有物理单位。没有 α、电导率或 Tauc 带隙。", "O eixo não tem unidade física. Não há α, condutividade nem lacuna de Tauc."),
    }
}

fn kv(out: &mut String, key: &str, value: &str, important: bool) {
    out.push_str(if important { "<tr class=\"key\">" } else { "<tr>" });
    out.push_str("<td class=\"text\">");
    out.push_str(&esc(key));
    out.push_str("</td><td>");
    out.push_str(&esc(value));
    out.push_str("</td></tr>");
}

fn cell(out: &mut String, value: &str) {
    out.push_str("<td>");
    out.push_str(&esc(value));
    out.push_str("</td>");
}

fn cell_text(out: &mut String, value: &str) {
    out.push_str("<td class=\"text\">");
    out.push_str(&esc(value));
    out.push_str("</td>");
}

fn item(out: &mut String, text: &str) {
    out.push_str("<li>");
    out.push_str(&esc(text));
    out.push_str("</li>");
}

fn fmt(v: f64) -> String {
    if !v.is_finite() {
        return "—".into();
    }
    let a = v.abs();
    if a != 0.0 && (a >= 1.0e5 || a < 1.0e-3) {
        return format!("{v:.6e}");
    }
    let s = format!("{v:.6}");
    if let Some(stripped) = s.strip_suffix('0') {
        let trimmed = stripped.trim_end_matches('0').trim_end_matches('.');
        if trimmed.is_empty() { "0".into() } else { trimmed.to_string() }
    } else {
        s
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fit::{fit_text, TextFit};

    #[test]
    fn informe_lists_the_fit() {
        let mut opts = TextFit::default();
        opts.measure = "eps2".into();
        opts.unit = "cm".into();
        opts.iter_max = 8;
        let report = fit_text("100 0.2\n200 1.4\n300 0.8\n400 0.3\n500 0.1\n", &opts).expect("ajuste");
        let page = html(&report, "prueba.dat");
        assert!(page.contains("prueba.dat"));
        assert!(page.contains("χ²"));
        assert!(page.contains("<table"));
        assert!(page.contains("Constantes ópticas"));
        assert!(page.contains("Kuzmenko"));
        assert!(page.contains("class=\"key\""));
    }
}
