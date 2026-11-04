//! Gráficos SVG del ajuste. El servidor dibuja la figura; el cliente no hidrata ECharts.

use crate::fit::FitReport;
use crate::measure::{high_end_maximum, Measure, Peak, Unit, HC_EV_NM};

pub struct Series<'a> {
    pub name: &'a str,
    pub color: &'a str,
    pub xs: &'a [f64],
    pub ys: &'a [f64],
}

pub fn line_chart(
    title: &str,
    y_label: &str,
    x_label: &str,
    series: &[Series<'_>],
    log_x: bool,
    band: Option<(&[f64], &[f64], &[f64])>,
) -> String {
    let width = 720.0;
    let height = 340.0;
    let left = 74.0;
    let right = 18.0;
    let top = 36.0;
    let bottom = 48.0;
    let mut xmin = f64::INFINITY;
    let mut xmax = f64::NEG_INFINITY;
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    for s in series {
        let n = s.xs.len().min(s.ys.len());
        for i in 0..n {
            if s.xs[i].is_finite() {
                xmin = xmin.min(s.xs[i]);
                xmax = xmax.max(s.xs[i]);
            }
            if s.ys[i].is_finite() {
                ymin = ymin.min(s.ys[i]);
                ymax = ymax.max(s.ys[i]);
            }
        }
    }
    if let Some((xs, lo, hi)) = band {
        let n = xs.len().min(lo.len()).min(hi.len());
        for i in 0..n {
            if xs[i].is_finite() {
                xmin = xmin.min(xs[i]);
                xmax = xmax.max(xs[i]);
            }
            for v in [lo[i], hi[i]] {
                if v.is_finite() {
                    ymin = ymin.min(v);
                    ymax = ymax.max(v);
                }
            }
        }
    }
    if !xmin.is_finite() || !ymin.is_finite() {
        return String::new();
    }
    if (xmax - xmin).abs() < 1e-12 {
        xmax = xmin + 1.0;
    }
    let nonneg = ymin >= 0.0;
    if (ymax - ymin).abs() < 1e-12 {
        let half = (ymax.abs() * 0.1).max(0.5);
        ymin -= half;
        ymax += half;
    }
    let pad_y = (ymax - ymin) * 0.08;
    ymin -= pad_y;
    ymax += pad_y;
    if nonneg {
        ymin = ymin.max(0.0);
    }
    let plot_w = width - left - right;
    let plot_h = height - top - bottom;
    let use_log = log_x && xmin > 0.0 && xmax > xmin;
    let ln_min = if use_log { xmin.ln() } else { 0.0 };
    let ln_span = if use_log { (xmax.ln() - ln_min).max(1e-12) } else { 1.0 };
    let mapx = |x: f64| {
        if use_log {
            left + (x.ln() - ln_min) / ln_span * plot_w
        } else {
            left + (x - xmin) / (xmax - xmin) * plot_w
        }
    };
    let mapy = |y: f64| top + (1.0 - (y - ymin) / (ymax - ymin)) * plot_h;

    // La escala va en el SVG: el cliente invierte el mapeo para leer el valor exacto bajo el mouse.
    let mut svg = format!(
        "<figure class=\"chart\"><figcaption>{}</figcaption><svg viewBox=\"0 0 {width} {height}\" role=\"img\" aria-label=\"{}\" data-xmin=\"{xmin}\" data-xmax=\"{xmax}\" data-log=\"{}\" data-ymin=\"{ymin}\" data-ymax=\"{ymax}\" data-left=\"{left}\" data-top=\"{top}\" data-pw=\"{plot_w}\" data-ph=\"{plot_h}\" data-xlabel=\"{}\"><rect width=\"720\" height=\"340\" fill=\"#07141b\"/>",
        escape(title),
        escape(title),
        if use_log { 1 } else { 0 },
        escape(x_label)
    );
    let y0 = mapy(0.0).clamp(top, top + plot_h);
    svg.push_str(&format!(
        "<line x1=\"{left}\" y1=\"{y0:.2}\" x2=\"{:.2}\" y2=\"{y0:.2}\" stroke=\"rgba(62,224,197,0.28)\"/>",
        left + plot_w
    ));
    let tick = |v: f64| -> String {
        if !v.is_finite() {
            return String::new();
        }
        if v == 0.0 {
            return "0".into();
        }
        let a = v.abs();
        if a >= 1.0e5 || a < 1.0e-3 {
            format!("{v:.2e}")
        } else if a >= 1000.0 || (a >= 10.0 && (v - v.round()).abs() < 1e-9) {
            format!("{v:.0}")
        } else if a >= 10.0 {
            format!("{v:.2}")
        } else {
            let s = format!("{v:.3}");
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    };
    for v in [ymax, (ymin + ymax) * 0.5, ymin] {
        let yy = mapy(v);
        svg.push_str(&format!(
            "<line x1=\"{left}\" y1=\"{yy:.2}\" x2=\"{:.2}\" y2=\"{yy:.2}\" stroke=\"rgba(143,184,176,0.16)\"/><text x=\"{:.0}\" y=\"{:.1}\" fill=\"#8fb8b0\" font-size=\"11\" text-anchor=\"end\" font-family=\"Segoe UI,sans-serif\">{}</text>",
            left + plot_w,
            left - 8.0,
            yy + 4.0,
            tick(v)
        ));
    }

    if let Some((xs, lo, hi)) = band {
        let n = xs.len().min(lo.len()).min(hi.len());
        let mut d = String::new();
        let mut top_pts = Vec::new();
        let mut bot_pts = Vec::new();
        for i in 0..n {
            if !xs[i].is_finite() || !lo[i].is_finite() || !hi[i].is_finite() || (use_log && xs[i] <= 0.0) {
                continue;
            }
            top_pts.push((mapx(xs[i]), mapy(hi[i])));
            bot_pts.push((mapx(xs[i]), mapy(lo[i])));
        }
        if top_pts.len() >= 2 {
            d.push_str(&format!("M{:.2},{:.2} ", top_pts[0].0, top_pts[0].1));
            for p in top_pts.iter().skip(1) {
                d.push_str(&format!("L{:.2},{:.2} ", p.0, p.1));
            }
            for p in bot_pts.iter().rev() {
                d.push_str(&format!("L{:.2},{:.2} ", p.0, p.1));
            }
            d.push('Z');
            svg.push_str(&format!(
                "<path d=\"{d}\" fill=\"rgba(62,224,197,0.18)\" stroke=\"none\"/>"
            ));
            svg.push_str(
                "<rect x=\"374\" y=\"10\" width=\"10\" height=\"10\" rx=\"2\" fill=\"rgba(62,224,197,0.45)\"/><text x=\"390\" y=\"19\" fill=\"#d7f6f0\" font-size=\"12\" font-family=\"Segoe UI,sans-serif\">±1σ</text>",
            );
        }
    }

    for (idx, s) in series.iter().enumerate() {
        let n = s.xs.len().min(s.ys.len());
        let mut d = String::new();
        let mut started = false;
        for i in 0..n {
            if !s.xs[i].is_finite() || !s.ys[i].is_finite() || (use_log && s.xs[i] <= 0.0) {
                started = false;
                continue;
            }
            let cmd = if started { "L" } else { "M" };
            started = true;
            d.push_str(&format!("{cmd}{:.2},{:.2} ", mapx(s.xs[i]), mapy(s.ys[i])));
        }
        svg.push_str(&format!(
            "<path d=\"{d}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.8\" stroke-linejoin=\"round\" data-series=\"{}\"/>",
            s.color,
            escape(s.name)
        ));
        let lx = left + idx as f64 * 150.0;
        svg.push_str(&format!(
            "<rect x=\"{lx:.0}\" y=\"10\" width=\"10\" height=\"10\" rx=\"2\" fill=\"{}\"/><text x=\"{:.0}\" y=\"19\" fill=\"#d7f6f0\" font-size=\"12\" font-family=\"Segoe UI,sans-serif\">{}</text>",
            s.color,
            lx + 16.0,
            escape(s.name)
        ));
    }

    let mut x_ticks = Vec::new();
    if use_log {
        let mut p = xmin.log10().ceil() as i32;
        let p_hi = xmax.log10().floor() as i32;
        if p > p_hi {
            x_ticks.push(xmin);
            x_ticks.push(xmax);
        } else {
            while p <= p_hi && x_ticks.len() < 8 {
                let x = 10f64.powi(p);
                if x >= xmin * 0.999 && x <= xmax * 1.001 {
                    x_ticks.push(x);
                }
                p += 1;
            }
        }
    } else {
        for i in 0..5 {
            x_ticks.push(xmin + (xmax - xmin) * i as f64 / 4.0);
        }
    }
    for x in x_ticks {
        let px = mapx(x);
        svg.push_str(&format!(
            "<text x=\"{px:.1}\" y=\"{}\" fill=\"#8fb8b0\" font-size=\"11\" text-anchor=\"middle\" font-family=\"Segoe UI,sans-serif\">{}</text>",
            height - 22.0,
            tick(x)
        ));
    }
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{}\" fill=\"#d7f6f0\" font-size=\"12\" text-anchor=\"middle\" font-family=\"Segoe UI,sans-serif\">{}{}</text>",
        left + plot_w / 2.0,
        height - 6.0,
        escape(x_label),
        if use_log { "  (escala log)" } else { "" }
    ));
    svg.push_str(&format!(
        "<text x=\"8\" y=\"28\" fill=\"#d7f6f0\" font-size=\"12\" font-family=\"Segoe UI,sans-serif\">{}</text></svg></figure>",
        escape(y_label)
    ));
    svg
}

fn num(v: f64) -> String {
    if !v.is_finite() {
        return "—".into();
    }
    let a = v.abs();
    if a != 0.0 && (a >= 1.0e5 || a < 1.0e-3) {
        format!("{v:.3e}")
    } else if a >= 100.0 {
        format!("{v:.1}")
    } else {
        format!("{v:.4}")
    }
}

fn band_of<'a>(xs: &'a [f64], ys: &[f64], err: &[f64]) -> Option<(Vec<f64>, Vec<f64>)> {
    if err.len() != ys.len() || ys.len() != xs.len() || ys.is_empty() {
        return None;
    }
    Some((
        ys.iter().zip(err).map(|(y, e)| y - e).collect(),
        ys.iter().zip(err).map(|(y, e)| y + e).collect(),
    ))
}

fn peaks_table(peaks: &[Peak], unit: Unit, what: &str) -> String {
    if peaks.is_empty() {
        return String::new();
    }
    let energy = unit.known();
    let total: f64 = peaks.iter().map(|p| p.area).sum();
    let mut rows = String::new();
    for (i, p) in peaks.iter().enumerate() {
        let lambda = unit
            .energy(p.position)
            .filter(|e| *e > 0.0)
            .map(|e| format!("<td class=\"mono\">{}</td>", num(HC_EV_NM / e)))
            .unwrap_or_default();
        let width = match p.fwhm {
            Some(f) => {
                let mev = if matches!(unit, Unit::Ev | Unit::Nm) { format!(" ({:.1} meV)", f * 1000.0) } else { String::new() };
                format!("{}{mev}", num(f))
            }
            None => "abierto".into(),
        };
        let share = if total > 0.0 { format!("{:.1} %", 100.0 * p.area / total) } else { "—".into() };
        rows.push_str(&format!(
            "<tr><td>{}</td><td class=\"mono\">{}</td>{lambda}<td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"mono\">{share}</td></tr>",
            i + 1,
            num(p.position),
            escape(&width),
            num(p.height),
            num(p.area),
        ));
    }
    let lambda_head = if energy { "<th>λ (nm)</th>" } else { "" };
    format!(
        "<div class=\"table-wrap\"><table class=\"peaks\"><caption>Picos de {what}</caption><thead><tr><th>#</th><th>{}</th>{lambda_head}<th>FWHM</th><th>Altura</th><th>Área</th><th>% del área</th></tr></thead><tbody>{rows}</tbody></table></div>",
        escape(unit.axis()),
    )
}

/// Enlaces de descarga. Con red apuntan al servidor; sin red el JS les pone el archivo generado aquí.
fn downloads(download_href: &str, has_peaks: bool) -> String {
    let offline = download_href == "#";
    let mut out = String::from("<div class=\"downloads\"><span class=\"downloads-label\">Descargar</span>");
    let items: [(&str, &str, &str); 5] = [
        ("dat", ".dat", "Tabla completa con comentarios: ω, ε₁, ε₂, n, k, R y lo demás"),
        ("csv", "CSV", "La misma tabla con cabecera, para hoja de cálculo"),
        ("ajuste", "Datos y ajuste", "Tus puntos, el modelo en cada uno y la diferencia"),
        ("picos", "Picos", "Posición, altura, ancho y área de cada pico"),
        ("json", "JSON", "Todo el informe, para procesarlo"),
    ];
    for (fmt, label, title) in items {
        if fmt == "picos" && !has_peaks {
            continue;
        }
        if offline {
            out.push_str(&format!(
                "<a class=\"btn btn-ghost btn-small\" href=\"#\" data-lm-file=\"{fmt}\" title=\"{}\">{label}</a>",
                escape(title)
            ));
        } else {
            out.push_str(&format!(
                "<a class=\"btn btn-ghost btn-small\" href=\"{}/{fmt}\" title=\"{}\">{label}</a>",
                escape(download_href),
                escape(title)
            ));
        }
    }
    out.push_str("<a class=\"btn btn-ghost btn-small\" href=\"#ajuste\">Ajustar otra vez</a>");
    out.push_str("</div>");
    out
}

/// Botón final: un solo HTML con el resumen, los picos y el espectro.
fn report_button(download_href: &str) -> String {
    let link = if download_href == "#" {
        "<a class=\"btn btn-primary\" href=\"#\" data-lm-file=\"informe\">Descargar todos los resultados</a>".to_string()
    } else {
        format!(
            "<a class=\"btn btn-primary\" href=\"{}/informe\">Descargar todos los resultados</a>",
            escape(download_href)
        )
    };
    format!(
        "<div class=\"report-download\"><div><p class=\"eyebrow\">Archivo completo</p><h3>Informe del ajuste</h3><p>Un solo HTML, con las condiciones, los resultados resaltados, los avisos, los picos y cada punto del espectro en tablas. Se abre en el navegador y se imprime.</p></div>{link}</div>"
    )
}

fn push_tauc_facts(items: &mut Vec<(String, String)>, report: &FitReport) {
    let direct = report.tauc_direct;
    let indirect = report.tauc_indirect;
    let rough_value = |g: f64, kind: &str| format!("{} eV · {kind}, orientativo", num(g));
    match report.tauc_show.as_str() {
        "direct" => {
            if let Some(g) = direct {
                items.push(("Brecha de Tauc".to_string(), format!("{} eV · directa", num(g))));
            }
        }
        "indirect" => {
            if let Some(g) = indirect {
                items.push(("Brecha de Tauc".to_string(), format!("{} eV · indirecta", num(g))));
            }
        }
        "both" => {
            if let Some(g) = direct {
                items.push(("Corte directo (Tauc)".to_string(), format!("{} eV", num(g))));
            }
            if let Some(g) = indirect {
                items.push(("Corte indirecto (Tauc)".to_string(), format!("{} eV", num(g))));
            }
        }
        "rough" => {
            let direct_r = report.tauc_direct_r2.unwrap_or(0.0);
            let indirect_r = report.tauc_indirect_r2.unwrap_or(0.0);
            if direct_r >= indirect_r {
                if let Some(g) = direct {
                    items.push(("Corte de Tauc".to_string(), rough_value(g, "directa")));
                }
            } else if let Some(g) = indirect {
                items.push(("Corte de Tauc".to_string(), rough_value(g, "indirecta")));
            }
        }
        _ => {
            if let Some(g) = direct {
                items.push(("Brecha directa (Tauc)".to_string(), format!("{} eV", num(g))));
            }
            if let Some(g) = indirect {
                items.push(("Brecha indirecta (Tauc)".to_string(), format!("{} eV", num(g))));
            }
        }
    }
}

fn push_peak_facts(items: &mut Vec<(String, String)>, report: &FitReport) {
    let top = report.peaks.iter().max_by(|a, b| a.height.partial_cmp(&b.height).unwrap_or(std::cmp::Ordering::Equal));
    if let Some((imax, ymax)) = high_end_maximum(&report.eps2) {
        let at = report.omega.get(imax).copied();
        let where_ = at.map(|v| format!("{} · al final del rango", num(v))).unwrap_or_else(|| "al final del rango".into());
        let span = report.omega.last().copied().unwrap_or(0.0) - report.omega.first().copied().unwrap_or(0.0);
        let separated = top.is_some_and(|p| {
            at.is_some_and(|v| span.abs() > 0.0 && (v - p.position).abs() > 0.05 * span.abs()) && ymax > 1.8 * p.height.max(1e-12)
        });
        if separated || top.is_none() {
            items.push(("Máximo de ε₂".to_string(), format!("{where_} · altura {}", num(ymax))));
            if let Some(p) = top.filter(|_| separated) {
                items.push(("Pico local de ε₂".to_string(), format!("{} · altura {}", num(p.position), num(p.height))));
            }
            return;
        }
    }
    if let Some(p) = top {
        items.push(("Pico mayor de ε₂".to_string(), format!("{} · altura {}", num(p.position), num(p.height))));
    }
}

fn facts(items: &[(String, String)]) -> String {
    if items.is_empty() {
        return String::new();
    }
    let body: String = items
        .iter()
        .map(|(k, v)| format!("<div><dt>{}</dt><dd class=\"mono\">{}</dd></div>", escape(k), escape(v)))
        .collect();
    format!("<dl class=\"facts\">{body}</dl>")
}

fn wrap(chart: String) -> String {
    if chart.is_empty() {
        chart
    } else {
        format!("<div class=\"chart-wrap\">{chart}</div>")
    }
}

fn warnings_markup(report: &FitReport) -> String {
    if report.warnings.is_empty() {
        String::new()
    } else {
        let items: String = report.warnings.iter().map(|w| format!("<li>{}</li>", escape(w))).collect();
        format!("<ul class=\"warnings\">{items}</ul>")
    }
}

fn luminescence_markup(source: &str, report: &FitReport, download_href: &str) -> String {
    let unit = report.options.unit;
    let x_label = unit.axis();
    let data = line_chart(
        "Luminiscencia — intensidad medida",
        "I",
        x_label,
        &[Series { name: "I", color: "#3ee0c5", xs: &report.data_x, ys: &report.data_y }],
        false,
        None,
    );
    let vrs = if report.vrs_x.is_empty() {
        String::new()
    } else {
        line_chart(
            &format!("Absorción relativa por van Roosbroeck–Shockley, T = {:.0} K", report.options.temperature_k),
            "α / α máx",
            x_label,
            &[Series { name: "α relativa", color: "#f2c14e", xs: &report.vrs_x, ys: &report.vrs_y }],
            false,
            None,
        )
    };
    let mut items = Vec::new();
    items.push(("Picos".to_string(), report.peaks.len().to_string()));
    if let Some(c) = report.centroid {
        let lambda = unit.energy(c).filter(|e| *e > 0.0).map(|e| format!(" · {} nm", num(HC_EV_NM / e))).unwrap_or_default();
        items.push(("Centroide".to_string(), format!("{}{lambda}", num(c))));
    }
    if let Some(i) = report.integral {
        items.push(("Intensidad integrada".to_string(), num(i)));
    }
    if let Some(t) = report.carrier_temp {
        items.push(("T de la cola alta".to_string(), format!("{} K", num(t))));
    }
    let nm_note = if unit == Unit::Nm {
        "<li><strong>nm → eV</strong> El eje pasa a energía con E = 1239.84/λ, y la intensidad se multiplica por λ²/hc para que sea por intervalo de energía. Así los anchos y las áreas se comparan entre picos.</li>"
    } else {
        ""
    };
    format!(
        "<section class=\"panel results\"><div class=\"result-head\"><div><p class=\"eyebrow\">Resultado · luminiscencia</p><h2>{n} picos</h2><p class=\"muted\">{meta}</p><p>La luminiscencia es emisión: no hay Kramers–Kronig que darle. Salen los picos, sus anchos y áreas, el centroide y, con unidades de energía, la temperatura de la cola alta y la forma de la absorción.</p></div>{downloads}</div>{warns}{facts}{table}{data}{vrs}<ul class=\"glossary\"><li><strong>Pico</strong> Máximo local cuya prominencia (altura sobre el valle más alto a sus lados) pasa del 5 % del rango. Se listan los 12 más prominentes.</li><li><strong>FWHM</strong> Ancho a media altura sobre esa base local, interpolado. «Abierto» si la mitad no se alcanza dentro del rango.</li><li><strong>Área</strong> Integral del pico sobre su base, por trapecios; el porcentaje es sobre la suma de los picos.</li><li><strong>Centroide</strong> Σ x·I / Σ I del espectro entero.</li>{nm_note}<li><strong>T de la cola alta</strong> Del lado de alta energía, I ∝ E²·α·e^(−E/kT). Si α casi no cambia allí, la pendiente de ln(I/E²) da T. Es la temperatura efectiva de los portadores; mayor que la de la muestra si están calientes.</li><li><strong>van Roosbroeck–Shockley</strong> En equilibrio, α ∝ I·e^(E/kT)/E². Da la forma de la absorción, sin escala, con la T de la muestra que escribiste. Por encima del borde amplifica el ruido.</li></ul>{informe}</section>",
        n = report.peaks.len(),
        meta = escape(&format!("{} puntos · {} · {}", report.points_used, unit.label(), source)),
        downloads = downloads(download_href, !report.peaks.is_empty()),
        warns = warnings_markup(report),
        facts = facts(&items),
        table = peaks_table(&report.peaks, unit, "la intensidad"),
        data = wrap(data),
        vrs = wrap(vrs),
        informe = report_button(download_href),
    )
}

pub fn result_markup(source: &str, report: &FitReport, download_href: &str) -> String {
    let measure = report.measure();
    if measure == Measure::Lum {
        return luminescence_markup(source, report, download_href);
    }
    let unit = report.options.unit;
    let x_label = unit.axis();
    let sym = measure.symbol();
    let w0 = report.data_x.first().copied().unwrap_or(1.0);
    let w1 = report.data_x.last().copied().unwrap_or(1.0);
    let log_x = report.options.log_mesh || (w0 > 0.0 && w1 / w0 > 20.0);
    let (fit_x, fit_y): (&[f64], &[f64]) = if report.model_q.len() == report.data_x.len() {
        (&report.data_x, &report.model_q)
    } else if report.model_eps2.len() == report.data_x.len() {
        (&report.data_x, &report.model_eps2)
    } else {
        (&report.omega, &report.eps2)
    };
    let q_err: &[f64] = if report.q_err.len() == fit_y.len() { &report.q_err } else { &report.eps2_err };
    let q_band = band_of(fit_x, fit_y, q_err);
    let fit_panel = line_chart(
        &format!("{} — datos y ajuste", measure.label()),
        sym,
        x_label,
        &[
            Series { name: "datos", color: "#9aa7b2", xs: &report.data_x, ys: &report.data_y },
            Series { name: "ajuste", color: "#3ee0c5", xs: fit_x, ys: fit_y },
        ],
        log_x,
        q_band.as_ref().map(|(lo, hi)| (fit_x, lo.as_slice(), hi.as_slice())),
    );
    let eps2_panel = if measure == Measure::Eps2 {
        String::new()
    } else {
        let band = band_of(&report.omega, &report.eps2, &report.coef_err);
        line_chart(
            &format!("ε₂ recuperada de {sym}"),
            "ε₂",
            x_label,
            &[Series { name: "ε₂", color: "#3ee0c5", xs: &report.omega, ys: &report.eps2 }],
            log_x,
            band.as_ref().map(|(lo, hi)| (report.omega.as_slice(), lo.as_slice(), hi.as_slice())),
        )
    };
    let eps1 = line_chart(
        "ε₁ — parte real, Kramers–Kronig",
        "ε₁",
        x_label,
        &[Series { name: "ε₁", color: "#f2c14e", xs: &report.omega, ys: &report.eps1 }],
        log_x,
        None,
    );
    let diff = if report.difference.len() == report.data_x.len() {
        line_chart(
            "Diferencia (ajuste − datos) × 100",
            "Δ × 100",
            x_label,
            &[Series { name: "Δ × 100", color: "#7aa2ff", xs: &report.data_x, ys: &report.difference }],
            log_x,
            None,
        )
    } else {
        String::new()
    };
    let nk = line_chart(
        "n índice de refracción · k extinción",
        "n, k",
        x_label,
        &[
            Series { name: "n", color: "#3ee0c5", xs: &report.omega, ys: &report.n_index },
            Series { name: "k", color: "#7aa2ff", xs: &report.omega, ys: &report.k_index },
        ],
        log_x,
        None,
    );
    let refl = line_chart(
        "R de un medio semi-infinito, a incidencia normal",
        "R",
        x_label,
        &[Series { name: "R", color: "#3ee0c5", xs: &report.omega, ys: &report.reflectivity }],
        log_x,
        None,
    );
    let loss = if report.loss.len() == report.omega.len() && !report.loss.is_empty() {
        line_chart(
            "Función de pérdida −Im(1/ε)",
            "−Im(1/ε)",
            x_label,
            &[Series { name: "pérdida", color: "#f28b82", xs: &report.omega, ys: &report.loss }],
            log_x,
            None,
        )
    } else {
        String::new()
    };
    let (alpha_panel, sigma_panel, sigma2_panel) = if report.absorption.len() == report.omega.len() && !report.absorption.is_empty() {
        (
            line_chart(
                "α coeficiente de absorción, 4πν̃k",
                "α (cm⁻¹)",
                x_label,
                &[Series { name: "α", color: "#f2c14e", xs: &report.omega, ys: &report.absorption }],
                log_x,
                None,
            ),
            line_chart(
                "σ₁ conductividad óptica, ν̃ε₂/60",
                "σ₁ (Ω⁻¹cm⁻¹)",
                x_label,
                &[Series { name: "σ₁", color: "#7aa2ff", xs: &report.omega, ys: &report.sigma1 }],
                log_x,
                None,
            ),
            if report.sigma2.len() == report.omega.len() {
                line_chart(
                    "σ₂ conductividad óptica, ν̃(ε∞ − ε₁)/60",
                    "σ₂ (Ω⁻¹cm⁻¹)",
                    x_label,
                    &[Series { name: "σ₂", color: "#c084fc", xs: &report.omega, ys: &report.sigma2 }],
                    log_x,
                    None,
                )
            } else {
                String::new()
            },
        )
    } else {
        (String::new(), String::new(), String::new())
    };
    let (trans_panel, absorb_panel) = if report.transmittance.len() == report.omega.len() && !report.transmittance.is_empty() {
        (
            line_chart(
                "T transmitancia de una lámina en aire",
                "T",
                x_label,
                &[Series { name: "T", color: "#3ee0c5", xs: &report.omega, ys: &report.transmittance }],
                log_x,
                None,
            ),
            line_chart(
                "A absorbancia, −log₁₀(T)",
                "A",
                x_label,
                &[Series { name: "A", color: "#f2c14e", xs: &report.omega, ys: &report.absorbance }],
                log_x,
                None,
            ),
        )
    } else {
        (String::new(), String::new())
    };
    let ext = if report.ext_omega.is_empty() {
        String::new()
    } else {
        line_chart(
            "Fuera del rango medido. No son datos nuevos",
            "ε",
            x_label,
            &[
                Series { name: "ε₂", color: "#3ee0c5", xs: &report.ext_omega, ys: &report.ext_eps2 },
                Series { name: "ε₁", color: "#f2c14e", xs: &report.ext_omega, ys: &report.ext_eps1 },
            ],
            log_x,
            None,
        )
    };
    let budget = report.options.iter_max.clamp(1, 240);
    let steps = if report.converged {
        report.iters_used.saturating_sub(1).max(1).min(budget)
    } else {
        report.iters_used.min(budget)
    };
    let status = if report.converged && report.chisq < 1e-8 {
        format!("Usó {steps} de {budget} pasos. Paró porque el objetivo ya no bajaba un 1 % y los puntos ya quedan cerca.")
    } else if report.converged {
        format!(
            "Usó {steps} de {budget} pasos. Paró porque el objetivo ya no bajaba un 1 % y ningún punto queda a más del 2 % del rango."
        )
    } else if report.iters_used == 0 {
        "Sin pasos registrados.".into()
    } else if steps >= budget {
        format!("Llegó a los {budget} pasos y algún punto sigue lejos. En pantalla está el mejor espectro aceptado.")
    } else {
        format!("Paró a los {steps} de {budget} pasos. En pantalla está el mejor espectro aceptado.")
    };
    let dof_n = report.points_used.saturating_sub(report.mfit);
    let score = if dof_n == 0 {
        format!("χ² <span class=\"mono\">{:.4e}</span>", report.chisq)
    } else {
        format!(
            "χ² <span class=\"mono\">{:.4e}</span> · χ²/dof <span class=\"mono\">{:.4e}</span>",
            report.chisq, report.chisq_dof
        )
    };
    let dof_help = if dof_n == 0 {
        format!("Hay un coeficiente por punto, así que χ²/dof no aplica. χ² cerca de cero quiere decir que el ajuste pasa por tus datos de {sym}.")
    } else {
        format!("χ²/dof parte ese valor por los puntos que sobran. Si el error típico pasa de la mitad del |{sym}| medio, el modelo queda lejos.")
    };
    let mut items = Vec::new();
    items.push(("Medida".to_string(), measure.label().to_string()));
    if let Some(v) = report.eps_inf {
        let err = report.eps_inf_err.map(|e| format!(" ± {}", num(e))).unwrap_or_default();
        let note = if report.eps_inf_note.is_empty() { String::new() } else { format!(" · {}", report.eps_inf_note) };
        items.push(("ε∞".to_string(), format!("{}{err}{note}", num(v))));
    }
    if report.options.thickness_um > 0.0 {
        items.push(("Espesor".to_string(), format!("{} µm", num(report.options.thickness_um))));
    }
    push_tauc_facts(&mut items, report);
    push_peak_facts(&mut items, report);
    let measure_note = match measure {
        Measure::Eps2 => "La columna que pegaste, tratada como ε₂.".to_string(),
        Measure::Lum => String::new(),
        _ => format!(
            "La columna que pegaste es {sym}. El ajuste no la toca: busca el ε₂ cuyo par Kramers–Kronig (ε₁, ε₂) reproduce {sym}, y de ahí salen todos los paneles."
        ),
    };
    let units_note = if unit.known() {
        "<li><strong>α</strong> 4πν̃k en cm⁻¹. <strong>σ₁</strong> ν̃ε₂/60 y <strong>σ₂</strong> ν̃(ε∞ − ε₁)/60, en Ω⁻¹cm⁻¹. <strong>T</strong> y <strong>A</strong> salen si escribiste el espesor: T es la transmitancia de una lámina en aire, con las reflexiones múltiples incoherentes, y A = −log₁₀(T). <strong>Tauc</strong> Corte de la tangente más empinada de (αE)² (directa) o (αE)^½ (indirecta), antes del máximo. Se muestra la fórmula que es una recta. La otra, si no lo es, no es una segunda brecha.</li>"
    } else {
        "<li><strong>Unidades</strong> Con ω en cm⁻¹, eV o nm salen además α, σ₁, σ₂ y la brecha de Tauc. T y A piden además el espesor.</li>"
    };
    format!(
        "<section class=\"panel results\"><div class=\"result-head\"><div><p class=\"eyebrow\">Resultado · {label}</p><h2>{score}</h2><p class=\"muted\">{meta}</p><p>{status}</p></div>{downloads}</div>{warns}{facts}{table}{fit}{eps2}{eps1}{diff}{nk}{refl}{loss}{alpha}{sigma}{sigma2}{trans}{absorb}{ext}<ul class=\"glossary\"><li><strong>χ²</strong> Suma de ((dato − modelo) / error)², solo de {sym}. Sin tercera columna el error vale 1. La suavidad y el castigo de ε₂ negativa no entran en este número; sí entran en la parada.</li><li><strong>χ²/dof</strong> {dof_help}</li><li><strong>±1σ</strong> Barra de confianza. Es la raíz de la diagonal de la covarianza, la inversa de la curvatura de los datos. Aparece cuando pegas el error de cada punto. La suavidad no entra en esa barra.</li><li><strong>Datos</strong> <span class=\"swatch swatch-data\"></span> {measure_note}</li><li><strong>ε₂</strong> <span class=\"swatch swatch-e2\"></span> Lo que ajusta la base triangular. En los extremos medidos puede valer lo que mediste; un poco más allá baja en línea recta hasta cero.</li><li><strong>ε₁</strong> <span class=\"swatch swatch-e1\"></span> ε∞ más la Kramers–Kronig de ε₂, incluida esa cola. No es un segundo ajuste.</li><li><strong>n</strong> índice de refracción. <strong>k</strong> extinción, con el signo de ε₂, de modo que n² − k² = ε₁ y 2nk = ε₂. <strong>R</strong> reflectividad de un medio semi-infinito a incidencia normal, calculada con esos n y k. No es la de una película sobre un sustrato. <strong>−Im(1/ε)</strong> función de pérdida: sus picos marcan los modos longitudinales y el plasmón.</li>{units_note}<li><strong>Picos</strong> Máximos locales de ε₂ cuya prominencia pasa del 5 % del rango. Si ε₂ sigue subiendo al final de la ventana, ese máximo es el borde de absorción y no entra como pico cerrado.</li><li><strong>Gráficas</strong> Pasa el mouse para leer el valor exacto del punto más cercano de cada curva. Cada gráfica se baja en SVG o PNG con los botones de debajo.</li></ul>{informe}</section>",
        label = escape(measure.symbol()),
        meta = escape(&format!("{} puntos · {} coeficientes · {} · {}", report.points_used, report.mfit, unit.label(), source)),
        downloads = downloads(download_href, !report.peaks.is_empty()),
        status = escape(&status),
        warns = warnings_markup(report),
        facts = facts(&items),
        table = peaks_table(&report.peaks, unit, "ε₂"),
        dof_help = escape(&dof_help),
        measure_note = escape(&measure_note),
        fit = wrap(fit_panel),
        eps2 = wrap(eps2_panel),
        eps1 = wrap(eps1),
        diff = wrap(diff),
        nk = wrap(nk),
        refl = wrap(refl),
        loss = wrap(loss),
        alpha = wrap(alpha_panel),
        sigma = wrap(sigma_panel),
        sigma2 = wrap(sigma2_panel),
        trans = wrap(trans_panel),
        absorb = wrap(absorb_panel),
        ext = wrap(ext),
        informe = report_button(download_href),
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
