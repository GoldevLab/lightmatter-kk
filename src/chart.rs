//! Gráficos SVG del ajuste. El servidor dibuja la figura; el cliente no hidrata ECharts.

use crate::fit::FitReport;
use crate::i18n::{tx, Lang};
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
        "<figure class=\"chart\"><figcaption>{}</figcaption><svg viewBox=\"0 0 {width} {height}\" role=\"img\" aria-label=\"{}\" data-xmin=\"{xmin}\" data-xmax=\"{xmax}\" data-log=\"{}\" data-ymin=\"{ymin}\" data-ymax=\"{ymax}\" data-left=\"{left}\" data-top=\"{top}\" data-pw=\"{plot_w}\" data-ph=\"{plot_h}\" data-xlabel=\"{}\"><rect width=\"720\" height=\"340\" fill=\"var(--chart-bg)\"/>",
        escape(title),
        escape(title),
        if use_log { 1 } else { 0 },
        escape(x_label)
    );
    let y0 = mapy(0.0).clamp(top, top + plot_h);
    svg.push_str(&format!(
        "<line x1=\"{left}\" y1=\"{y0:.2}\" x2=\"{:.2}\" y2=\"{y0:.2}\" stroke=\"var(--chart-axis)\"/>",
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
            "<line x1=\"{left}\" y1=\"{yy:.2}\" x2=\"{:.2}\" y2=\"{yy:.2}\" stroke=\"var(--chart-grid)\"/><text x=\"{:.0}\" y=\"{:.1}\" fill=\"var(--chart-ink)\" font-size=\"11\" text-anchor=\"end\" font-family=\"Segoe UI,sans-serif\">{}</text>",
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
                "<path d=\"{d}\" fill=\"var(--chart-band)\" stroke=\"none\"/>"
            ));
            svg.push_str(
                "<rect x=\"374\" y=\"10\" width=\"10\" height=\"10\" rx=\"2\" fill=\"rgba(62,224,197,0.45)\"/><text x=\"390\" y=\"19\" fill=\"var(--chart-label)\" font-size=\"12\" font-family=\"Segoe UI,sans-serif\">±1σ</text>",
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
            "<rect x=\"{lx:.0}\" y=\"10\" width=\"10\" height=\"10\" rx=\"2\" fill=\"{}\"/><text x=\"{:.0}\" y=\"19\" fill=\"var(--chart-label)\" font-size=\"12\" font-family=\"Segoe UI,sans-serif\">{}</text>",
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
            "<text x=\"{px:.1}\" y=\"{}\" fill=\"var(--chart-ink)\" font-size=\"11\" text-anchor=\"middle\" font-family=\"Segoe UI,sans-serif\">{}</text>",
            height - 22.0,
            tick(x)
        ));
    }
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{}\" fill=\"var(--chart-label)\" font-size=\"12\" text-anchor=\"middle\" font-family=\"Segoe UI,sans-serif\">{}{}</text>",
        left + plot_w / 2.0,
        height - 6.0,
        escape(x_label),
        if use_log { "  (escala log)" } else { "" }
    ));
    svg.push_str(&format!(
        "<text x=\"8\" y=\"28\" fill=\"var(--chart-label)\" font-size=\"12\" font-family=\"Segoe UI,sans-serif\">{}</text></svg></figure>",
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

fn peaks_table(peaks: &[Peak], unit: Unit, what: &str, lang: Lang) -> String {
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
            None => tx(lang, "abierto", "open", "未闭合", "aberto").into(),
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
        "<div class=\"table-wrap\"><table class=\"peaks\"><caption>{} {what}</caption><thead><tr><th>#</th><th>{}</th>{lambda_head}<th>FWHM</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>{rows}</tbody></table></div>",
        tx(lang, "Picos de", "Peaks of", "峰：", "Picos de"),
        escape(unit.axis()),
        tx(lang, "Altura", "Height", "高度", "Altura"),
        tx(lang, "Área", "Area", "面积", "Área"),
        tx(lang, "% del área", "% of area", "面积占比", "% da área"),
    )
}

/// Enlaces de descarga. Con red apuntan al servidor; sin red el JS les pone el archivo generado aquí.
fn downloads(download_href: &str, has_peaks: bool, lang: Lang) -> String {
    let offline = download_href == "#";
    let mut out = format!(
        "<div class=\"downloads\"><span class=\"downloads-label\">{}</span>",
        tx(lang, "Descargar", "Download", "下载", "Baixar")
    );
    let items: [(&str, &str, &str); 5] = [
        ("dat", ".dat", tx(lang, "Tabla completa con comentarios: ω, ε₁, ε₂, n, k, R y lo demás", "Full table with comments: ω, ε₁, ε₂, n, k, R, and the rest", "带注释的完整表：ω、ε₁、ε₂、n、k、R 及其他", "Tabela completa com comentários: ω, ε₁, ε₂, n, k, R e o resto")),
        ("csv", "CSV", tx(lang, "La misma tabla con cabecera, para hoja de cálculo", "The same table with a header, for a spreadsheet", "同一张表，带表头，用于电子表格", "A mesma tabela com cabeçalho, para uma planilha")),
        ("ajuste", tx(lang, "Datos y ajuste", "Data and fit", "数据与拟合", "Dados e ajuste"), tx(lang, "Tus puntos, el modelo en cada uno y la diferencia", "Your points, the model at each one, and the difference", "你的点、每点的模型和差值", "Os seus pontos, o modelo em cada um e a diferença")),
        ("picos", tx(lang, "Picos", "Peaks", "峰", "Picos"), tx(lang, "Posición, altura, ancho y área de cada pico", "Position, height, width, and area of each peak", "每个峰的位置、高度、宽度和面积", "Posição, altura, largura e área de cada pico")),
        ("json", "JSON", tx(lang, "Todo el informe, para procesarlo", "The whole report, for processing", "完整报告，便于处理", "O relatório inteiro, para processar")),
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
    out.push_str(&format!(
        "<a class=\"btn btn-ghost btn-small\" href=\"#ajuste\">{}</a>",
        tx(lang, "Ajustar otra vez", "Fit again", "再拟合一次", "Ajustar de novo")
    ));
    out.push_str("</div>");
    out
}

/// Botón final: un solo HTML con el resumen, los picos y el espectro.
fn report_button(download_href: &str, lang: Lang) -> String {
    let label = tx(lang, "Descargar todos los resultados", "Download all results", "下载全部结果", "Baixar todos os resultados");
    let link = if download_href == "#" {
        format!("<a class=\"btn btn-primary\" href=\"#\" data-lm-file=\"informe\">{label}</a>")
    } else {
        format!(
            "<a class=\"btn btn-primary\" href=\"{}/informe\">{label}</a>",
            escape(download_href)
        )
    };
    format!(
        "<div class=\"report-download\"><div><p class=\"eyebrow\">{}</p><h3>{}</h3><p>{}</p></div>{link}</div>",
        tx(lang, "Archivo completo", "Complete file", "完整文件", "Arquivo completo"),
        tx(lang, "Informe del ajuste", "Fit report", "拟合报告", "Relatório do ajuste"),
        tx(lang, "Un solo HTML, con las condiciones, los resultados resaltados, los avisos, los picos y cada punto del espectro en tablas. Se abre en el navegador y se imprime.", "One HTML file, with the conditions, the highlighted results, the warnings, the peaks, and every spectrum point in tables. It opens in the browser and prints.", "一个 HTML 文件，含条件、突出的结果、提示、峰，以及表格中的每个光谱点。可在浏览器中打开并打印。", "Um só HTML, com as condições, os resultados em destaque, os avisos, os picos e cada ponto do espectro em tabelas. Abre no navegador e imprime."),
    )
}

fn push_tauc_facts(items: &mut Vec<(String, String)>, report: &FitReport, lang: Lang) {
    let direct = report.tauc_direct;
    let indirect = report.tauc_indirect;
    let gap = tx(lang, "Brecha de Tauc", "Tauc gap", "Tauc 带隙", "Lacuna de Tauc");
    let direct_cut = tx(lang, "Corte directo (Tauc)", "Direct edge (Tauc)", "直接带边（Tauc）", "Corte direto (Tauc)");
    let indirect_cut = tx(lang, "Corte indirecto (Tauc)", "Indirect edge (Tauc)", "间接带边（Tauc）", "Corte indireto (Tauc)");
    let rough_cut = tx(lang, "Corte de Tauc", "Tauc edge", "Tauc 带边", "Corte de Tauc");
    let direct_gap = tx(lang, "Brecha directa (Tauc)", "Direct gap (Tauc)", "直接带隙（Tauc）", "Lacuna direta (Tauc)");
    let indirect_gap = tx(lang, "Brecha indirecta (Tauc)", "Indirect gap (Tauc)", "间接带隙（Tauc）", "Lacuna indireta (Tauc)");
    let directa = tx(lang, "directa", "direct", "直接", "direta");
    let indirecta = tx(lang, "indirecta", "indirect", "间接", "indireta");
    let rough = tx(lang, "orientativo", "indicative", "仅供参考", "indicativo");
    match report.tauc_show.as_str() {
        "direct" => {
            if let Some(g) = direct {
                items.push((gap.into(), format!("{} eV · {directa}", num(g))));
            }
        }
        "indirect" => {
            if let Some(g) = indirect {
                items.push((gap.into(), format!("{} eV · {indirecta}", num(g))));
            }
        }
        "both" => {
            if let Some(g) = direct {
                items.push((direct_cut.into(), format!("{} eV", num(g))));
            }
            if let Some(g) = indirect {
                items.push((indirect_cut.into(), format!("{} eV", num(g))));
            }
        }
        "rough" => {
            let direct_r = report.tauc_direct_r2.unwrap_or(0.0);
            let indirect_r = report.tauc_indirect_r2.unwrap_or(0.0);
            if direct_r >= indirect_r {
                if let Some(g) = direct {
                    items.push((rough_cut.into(), format!("{} eV · {directa}, {rough}", num(g))));
                }
            } else if let Some(g) = indirect {
                items.push((rough_cut.into(), format!("{} eV · {indirecta}, {rough}", num(g))));
            }
        }
        _ => {
            if let Some(g) = direct {
                items.push((direct_gap.into(), format!("{} eV", num(g))));
            }
            if let Some(g) = indirect {
                items.push((indirect_gap.into(), format!("{} eV", num(g))));
            }
        }
    }
}

fn push_peak_facts(items: &mut Vec<(String, String)>, report: &FitReport, lang: Lang) {
    let top = report.peaks.iter().max_by(|a, b| a.height.partial_cmp(&b.height).unwrap_or(std::cmp::Ordering::Equal));
    if let Some((imax, ymax)) = high_end_maximum(&report.eps2) {
        let at = report.omega.get(imax).copied();
        let end = tx(lang, "al final del rango", "at the end of the range", "在范围末端", "no fim da faixa");
        let where_ = at.map(|v| format!("{} · {end}", num(v))).unwrap_or_else(|| end.into());
        let span = report.omega.last().copied().unwrap_or(0.0) - report.omega.first().copied().unwrap_or(0.0);
        let separated = top.is_some_and(|p| {
            at.is_some_and(|v| span.abs() > 0.0 && (v - p.position).abs() > 0.05 * span.abs()) && ymax > 1.8 * p.height.max(1e-12)
        });
        if separated || top.is_none() {
            let height = tx(lang, "altura", "height", "高度", "altura");
            items.push((tx(lang, "Máximo de ε₂", "ε₂ maximum", "ε₂ 最大值", "Máximo de ε₂").into(), format!("{where_} · {height} {}", num(ymax))));
            if let Some(p) = top.filter(|_| separated) {
                items.push((tx(lang, "Pico local de ε₂", "Local ε₂ peak", "ε₂ 局部峰", "Pico local de ε₂").into(), format!("{} · {height} {}", num(p.position), num(p.height))));
            }
            return;
        }
    }
    if let Some(p) = top {
        let height = tx(lang, "altura", "height", "高度", "altura");
        items.push((tx(lang, "Pico mayor de ε₂", "Largest ε₂ peak", "ε₂ 最高峰", "Maior pico de ε₂").into(), format!("{} · {height} {}", num(p.position), num(p.height))));
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

fn luminescence_markup(source: &str, report: &FitReport, download_href: &str, lang: Lang) -> String {
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
        "<section class=\"panel results\"><div class=\"result-head\"><div><p class=\"eyebrow\">Resultado · luminiscencia</p><h2>{n} picos</h2><p class=\"muted\">{meta}</p><p>La luminiscencia es emisión: no hay Kramers–Kronig que darle. Salen los picos, sus anchos y áreas, el centroide y, con unidades de energía, la temperatura de la cola alta y la forma de la absorción.</p></div>{downloads}</div>{warns}{facts}{table}{data}{vrs}<ul class=\"glossary\"><li><strong>Pico</strong> Máximo local cuya prominencia (altura sobre el valle más alto a sus lados) pasa del 5 % del rango. Se listan los 12 más prominentes.</li><li><strong>FWHM</strong> Ancho a media altura sobre esa base local, interpolado. «Abierto» si la mitad no se alcanza dentro del rango.</li><li><strong>Área</strong> Integral del pico sobre su base, por trapecios; el porcentaje es sobre la suma de los picos.</li><li><strong>Centroide</strong> Σ x·I / Σ I del espectro entero.</li>{nm_note}<li><strong>T de la cola alta</strong> Del lado de alta energía, I ∝ E²·α·e^(−E/kT). Si α casi no cambia allí, la pendiente de ln(I/E²) da T. Es la temperatura efectiva de los portadores; mayor que la de la muestra si están calientes.</li><li><strong>van Roosbroeck–Shockley</strong> En equilibrio, α ∝ I·e^(E/kT)/E². Da la forma de la absorción, sin escala, con la T de la muestra que escribiste. Por encima del borde amplifica el ruido.</li></ul>{publish}{informe}</section>",
        n = report.peaks.len(),
        meta = escape(&format!("{} puntos · {} · {}", report.points_used, unit.label(), source)),
        downloads = downloads(download_href, !report.peaks.is_empty(), lang),
        warns = warnings_markup(report),
        facts = facts(&items),
        table = peaks_table(&report.peaks, unit, tx(lang, "la intensidad", "the intensity", "强度", "a intensidade"), lang),
        data = wrap(data),
        vrs = wrap(vrs),
        informe = report_button(download_href, lang),
        publish = crate::publish::form(download_href, report, lang),
    )
}

pub fn result_markup(source: &str, report: &FitReport, download_href: &str, lang: Lang) -> String {
    let measure = report.measure();
    if measure == Measure::Lum {
        return luminescence_markup(source, report, download_href, lang);
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
        &format!(
            "{} — {}",
            sym,
            tx(lang, "datos y ajuste", "data and fit", "数据与拟合", "dados e ajuste")
        ),
        sym,
        x_label,
        &[
            Series { name: tx(lang, "datos", "data", "数据", "dados"), color: "#9aa7b2", xs: &report.data_x, ys: &report.data_y },
            Series { name: tx(lang, "ajuste", "fit", "拟合", "ajuste"), color: "#3ee0c5", xs: fit_x, ys: fit_y },
        ],
        log_x,
        q_band.as_ref().map(|(lo, hi)| (fit_x, lo.as_slice(), hi.as_slice())),
    );
    let eps2_panel = if measure == Measure::Eps2 {
        String::new()
    } else {
        let band = band_of(&report.omega, &report.eps2, &report.coef_err);
        line_chart(
            &format!("{} {sym}", tx(lang, "ε₂ recuperada de", "ε₂ recovered from", "由以下量恢复的 ε₂", "ε₂ recuperada de")),
            "ε₂",
            x_label,
            &[Series { name: "ε₂", color: "#3ee0c5", xs: &report.omega, ys: &report.eps2 }],
            log_x,
            band.as_ref().map(|(lo, hi)| (report.omega.as_slice(), lo.as_slice(), hi.as_slice())),
        )
    };
    let eps1 = line_chart(
        tx(lang, "ε₁ — parte real, Kramers–Kronig", "ε₁ — real part, Kramers–Kronig", "ε₁ — 实部，克拉默斯–克罗尼格", "ε₁ — parte real, Kramers–Kronig"),
        "ε₁",
        x_label,
        &[Series { name: "ε₁", color: "#f2c14e", xs: &report.omega, ys: &report.eps1 }],
        log_x,
        None,
    );
    let diff = if report.difference.len() == report.data_x.len() {
        line_chart(
            tx(lang, "Diferencia (ajuste − datos) × 100", "Difference (fit − data) × 100", "差值（拟合 − 数据）× 100", "Diferença (ajuste − dados) × 100"),
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
        tx(lang, "n índice de refracción · k extinción", "n refractive index · k extinction", "n 折射率 · k 消光", "n índice de refração · k extinção"),
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
        tx(lang, "R de un medio semi-infinito, a incidencia normal", "R of a semi-infinite medium, at normal incidence", "半无限介质的正入射 R", "R de um meio semi-infinito, em incidência normal"),
        "R",
        x_label,
        &[Series { name: "R", color: "#3ee0c5", xs: &report.omega, ys: &report.reflectivity }],
        log_x,
        None,
    );
    let loss = if report.loss.len() == report.omega.len() && !report.loss.is_empty() {
        line_chart(
            tx(lang, "Función de pérdida −Im(1/ε)", "Loss function −Im(1/ε)", "损失函数 −Im(1/ε)", "Função de perda −Im(1/ε)"),
            "−Im(1/ε)",
            x_label,
                &[Series { name: tx(lang, "pérdida", "loss", "损失", "perda"), color: "#f28b82", xs: &report.omega, ys: &report.loss }],
            log_x,
            None,
        )
    } else {
        String::new()
    };
    let (alpha_panel, sigma_panel, sigma2_panel) = if report.absorption.len() == report.omega.len() && !report.absorption.is_empty() {
        (
            line_chart(
                tx(lang, "α coeficiente de absorción, 4πν̃k", "α absorption coefficient, 4πν̃k", "α 吸收系数，4πν̃k", "α coeficiente de absorção, 4πν̃k"),
                "α (cm⁻¹)",
                x_label,
                &[Series { name: "α", color: "#f2c14e", xs: &report.omega, ys: &report.absorption }],
                log_x,
                None,
            ),
            line_chart(
                tx(lang, "σ₁ conductividad óptica, ν̃ε₂/60", "σ₁ optical conductivity, ν̃ε₂/60", "σ₁ 光电导，ν̃ε₂/60", "σ₁ condutividade óptica, ν̃ε₂/60"),
                "σ₁ (Ω⁻¹cm⁻¹)",
                x_label,
                &[Series { name: "σ₁", color: "#7aa2ff", xs: &report.omega, ys: &report.sigma1 }],
                log_x,
                None,
            ),
            if report.sigma2.len() == report.omega.len() {
                line_chart(
                    tx(lang, "σ₂ conductividad óptica, ν̃(ε∞ − ε₁)/60", "σ₂ optical conductivity, ν̃(ε∞ − ε₁)/60", "σ₂ 光电导，ν̃(ε∞ − ε₁)/60", "σ₂ condutividade óptica, ν̃(ε∞ − ε₁)/60"),
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
                tx(lang, "T transmitancia de una lámina en aire", "T transmittance of a slab in air", "空气中薄片的透射率 T", "T transmitância de uma lâmina no ar"),
                "T",
                x_label,
                &[Series { name: "T", color: "#3ee0c5", xs: &report.omega, ys: &report.transmittance }],
                log_x,
                None,
            ),
            line_chart(
                tx(lang, "A absorbancia, −log₁₀(T)", "A absorbance, −log₁₀(T)", "吸光度 A，−log₁₀(T)", "A absorbância, −log₁₀(T)"),
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
            tx(lang, "Fuera del rango medido. No son datos nuevos", "Outside the measured range. These are not new data", "测量范围之外。不是新数据", "Fora da faixa medida. Não são dados novos"),
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
    let budget = report.options.iter_max.clamp(1, 1000);
    let steps = if report.converged {
        report.iters_used.saturating_sub(1).max(1).min(budget)
    } else {
        report.iters_used.min(budget)
    };
    let status = if report.converged && report.chisq < 1e-8 {
        match lang {
            Lang::Es => format!("Usó {steps} de {budget} pasos. Paró porque el objetivo ya no bajaba un 1 % y los puntos ya quedan cerca."),
            Lang::En => format!("Used {steps} of {budget} steps. It stopped because the objective no longer fell by 1% and the points are already close."),
            Lang::Zh => format!("用了 {budget} 步中的 {steps} 步。目标不再下降 1%，而且各点已经接近，所以停止。"),
            Lang::Pt => format!("Usou {steps} de {budget} passos. Parou porque o objetivo já não caía 1% e os pontos já estão perto."),
        }
    } else if report.converged {
        match lang {
            Lang::Es => format!("Usó {steps} de {budget} pasos. Paró porque el objetivo ya no bajaba un 1 % y ningún punto queda a más del 2 % del rango."),
            Lang::En => format!("Used {steps} of {budget} steps. It stopped because the objective no longer fell by 1% and no point is more than 2% of the range away."),
            Lang::Zh => format!("用了 {budget} 步中的 {steps} 步。目标不再下降 1%，且没有点偏离超过范围的 2%，所以停止。"),
            Lang::Pt => format!("Usou {steps} de {budget} passos. Parou porque o objetivo já não caía 1% e nenhum ponto fica a mais de 2% da faixa."),
        }
    } else if report.iters_used == 0 {
        tx(lang, "Sin pasos registrados.", "No steps recorded.", "没有记录步数。", "Nenhum passo registrado.").into()
    } else if steps >= budget {
        match lang {
            Lang::Es => format!("Llegó a los {budget} pasos y algún punto sigue lejos. En pantalla está el mejor espectro aceptado."),
            Lang::En => format!("It reached the {budget} steps and some point is still far. The screen shows the best accepted spectrum."),
            Lang::Zh => format!("已经到了 {budget} 步，仍有点偏远。屏幕上是已接受的最佳光谱。"),
            Lang::Pt => format!("Chegou aos {budget} passos e algum ponto ainda está longe. Na tela está o melhor espectro aceito."),
        }
    } else {
        match lang {
            Lang::Es => format!("Paró a los {steps} de {budget} pasos. En pantalla está el mejor espectro aceptado."),
            Lang::En => format!("It stopped at {steps} of {budget} steps. The screen shows the best accepted spectrum."),
            Lang::Zh => format!("在 {budget} 步中的第 {steps} 步停止。屏幕上是已接受的最佳光谱。"),
            Lang::Pt => format!("Parou nos {steps} de {budget} passos. Na tela está o melhor espectro aceito."),
        }
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
        match lang {
            Lang::Es => format!("Hay un coeficiente por punto, así que χ²/dof no aplica. χ² cerca de cero quiere decir que el ajuste pasa por tus datos de {sym}."),
            Lang::En => format!("There is one coefficient per point, so χ²/dof does not apply. χ² near zero means the fit passes through your {sym} data."),
            Lang::Zh => format!("每个点有一个系数，所以 χ²/dof 不适用。χ² 接近零表示拟合穿过你的 {sym} 数据。"),
            Lang::Pt => format!("Há um coeficiente por ponto, então χ²/dof não se aplica. χ² perto de zero quer dizer que o ajuste passa pelos seus dados de {sym}."),
        }
    } else {
        match lang {
            Lang::Es => format!("χ²/dof parte ese valor por los puntos que sobran. Si el error típico pasa de la mitad del |{sym}| medio, el modelo queda lejos."),
            Lang::En => format!("χ²/dof divides that value by the leftover points. If the typical error exceeds half the mean |{sym}|, the model is far."),
            Lang::Zh => format!("χ²/dof 把该值除以多余的点数。若典型误差超过平均 |{sym}| 的一半，模型就偏远。"),
            Lang::Pt => format!("χ²/dof divide esse valor pelos pontos que sobram. Se o erro típico passa da metade do |{sym}| médio, o modelo fica longe."),
        }
    };
    let mut items = Vec::new();
    items.push((
        tx(lang, "Medida", "Measurement", "测量", "Medida").to_string(),
        crate::i18n::measure_name(lang, measure.key()).to_string(),
    ));
    if let Some(v) = report.eps_inf {
        let err = report.eps_inf_err.map(|e| format!(" ± {}", num(e))).unwrap_or_default();
        let note = if report.eps_inf_note.is_empty() { String::new() } else { format!(" · {}", report.eps_inf_note) };
        items.push(("ε∞".to_string(), format!("{}{err}{note}", num(v))));
    }
    if report.options.thickness_um > 0.0 {
        items.push((tx(lang, "Espesor", "Thickness", "厚度", "Espessura").to_string(), format!("{} µm", num(report.options.thickness_um))));
    }
    push_tauc_facts(&mut items, report, lang);
    push_peak_facts(&mut items, report, lang);
    let measure_note = match measure {
        Measure::Eps2 => tx(lang, "La columna que pegaste, tratada como ε₂.", "The column you pasted, treated as ε₂.", "你粘贴的列，按 ε₂ 处理。", "A coluna que você colou, tratada como ε₂.").to_string(),
        Measure::Lum => String::new(),
        _ => match lang {
            Lang::Es => format!("La columna que pegaste es {sym}. El ajuste no la toca: busca el ε₂ cuyo par Kramers–Kronig (ε₁, ε₂) reproduce {sym}, y de ahí salen todos los paneles."),
            Lang::En => format!("The column you pasted is {sym}. The fit does not edit it: it looks for the ε₂ whose Kramers–Kronig pair (ε₁, ε₂) reproduces {sym}, and every panel comes from that."),
            Lang::Zh => format!("你粘贴的列是 {sym}。拟合不改它：它寻找克拉默斯–克罗尼格配对 (ε₁, ε₂) 能再现 {sym} 的 ε₂，所有图都由此得到。"),
            Lang::Pt => format!("A coluna que você colou é {sym}. O ajuste não a altera: procura o ε₂ cujo par de Kramers–Kronig (ε₁, ε₂) reproduz {sym}, e daí saem todos os painéis."),
        },
    };
    let units_note = if unit.known() {
        tx(
            lang,
            "<li><strong>α</strong> 4πν̃k en cm⁻¹. <strong>σ₁</strong> ν̃ε₂/60 y <strong>σ₂</strong> ν̃(ε∞ − ε₁)/60, en Ω⁻¹cm⁻¹. <strong>T</strong> y <strong>A</strong> salen si escribiste el espesor: T es la transmitancia de una lámina en aire, con las reflexiones múltiples incoherentes, y A = −log₁₀(T). <strong>Tauc</strong> Corte de la tangente más empinada de (αE)² (directa) o (αE)^½ (indirecta), antes del máximo. Se muestra la fórmula que es una recta. La otra, si no lo es, no es una segunda brecha.</li>",
            "<li><strong>α</strong> 4πν̃k in cm⁻¹. <strong>σ₁</strong> ν̃ε₂/60 and <strong>σ₂</strong> ν̃(ε∞ − ε₁)/60, in Ω⁻¹cm⁻¹. <strong>T</strong> and <strong>A</strong> appear if you entered the thickness: T is the transmittance of a slab in air, with incoherent multiple reflections, and A = −log₁₀(T). <strong>Tauc</strong> Intercept of the steepest tangent of (αE)² (direct) or (αE)^½ (indirect), before the maximum. The formula that is a straight line is shown. The other one, if it is not, is not a second gap.</li>",
            "<li><strong>α</strong> 为 cm⁻¹ 下的 4πν̃k。<strong>σ₁</strong> 为 ν̃ε₂/60，<strong>σ₂</strong> 为 ν̃(ε∞ − ε₁)/60，单位 Ω⁻¹cm⁻¹。若填写了厚度，则给出 <strong>T</strong> 和 <strong>A</strong>：T 是空气中薄片的透射率，含非相干多次反射，A = −log₁₀(T)。<strong>Tauc</strong> 是最大值之前 (αE)²（直接）或 (αE)^½（间接）最陡切线的截距。只显示成直线的那个公式。另一个若不是直线，就不是第二个带隙。</li>",
            "<li><strong>α</strong> 4πν̃k em cm⁻¹. <strong>σ₁</strong> ν̃ε₂/60 e <strong>σ₂</strong> ν̃(ε∞ − ε₁)/60, em Ω⁻¹cm⁻¹. <strong>T</strong> e <strong>A</strong> aparecem se você escreveu a espessura: T é a transmitância de uma lâmina no ar, com reflexões múltiplas incoerentes, e A = −log₁₀(T). <strong>Tauc</strong> Corte da tangente mais íngreme de (αE)² (direta) ou (αE)^½ (indireta), antes do máximo. Mostra-se a fórmula que é uma reta. A outra, se não for, não é uma segunda lacuna.</li>",
        )
    } else {
        tx(
            lang,
            "<li><strong>Unidades</strong> Con ω en cm⁻¹, eV o nm salen además α, σ₁, σ₂ y la brecha de Tauc. T y A piden además el espesor.</li>",
            "<li><strong>Units</strong> With ω in cm⁻¹, eV, or nm, α, σ₁, σ₂, and the Tauc gap are also shown. T and A also need the thickness.</li>",
            "<li><strong>单位</strong> ω 为 cm⁻¹、eV 或 nm 时，还会给出 α、σ₁、σ₂ 和 Tauc 带隙。T 和 A 还需要厚度。</li>",
            "<li><strong>Unidades</strong> Com ω em cm⁻¹, eV ou nm também saem α, σ₁, σ₂ e a lacuna de Tauc. T e A pedem ainda a espessura.</li>",
        )
    };
    let g_chi = tx(lang, "Suma de ((dato − modelo) / error)², solo de", "Sum of ((data − model) / error)², only for", "((数据 − 模型) / 误差)² 之和，只计", "Soma de ((dado − modelo) / erro)², só de");
    let g_chi2 = tx(lang, "Sin tercera columna el error vale 1. La suavidad y el castigo de ε₂ negativa no entran en este número; sí entran en la parada.", "Without a third column the error is 1. Smoothness and the penalty on negative ε₂ are not in this number; they do enter the stopping rule.", "没有第三列时误差为 1。平滑和对负 ε₂ 的惩罚不计入这个数，但会计入停止条件。", "Sem terceira coluna o erro vale 1. A suavidade e a penalidade de ε₂ negativa não entram neste número; entram na parada.");
    let g_bar = tx(lang, "Barra de confianza. Es la raíz de la diagonal de la covarianza, la inversa de la curvatura de los datos. Aparece cuando pegas el error de cada punto. La suavidad no entra en esa barra.", "Confidence bar. It is the square root of the covariance diagonal, the inverse of the curvature of the data. It appears when you paste the error of each point. Smoothness is not in that bar.", "置信条。它是协方差对角元的平方根，即数据曲率的逆。粘贴每点误差时出现。平滑不进入这条。", "Barra de confiança. É a raiz da diagonal da covariância, o inverso da curvatura dos dados. Aparece quando você cola o erro de cada ponto. A suavidade não entra nessa barra.");
    let g_e2 = tx(lang, "Lo que ajusta la base triangular. En los extremos medidos puede valer lo que mediste; un poco más allá baja en línea recta hasta cero.", "What the triangular basis fits. At the measured ends it can equal what you measured; a little beyond that it falls in a straight line to zero.", "三角基底所拟合的量。在测量端点可以等于你测到的值；再往外则直线降到零。", "O que a base triangular ajusta. Nos extremos medidos pode valer o que você mediu; um pouco além disso desce em linha reta até zero.");
    let g_e1 = tx(lang, "ε∞ más la Kramers–Kronig de ε₂, incluida esa cola. No es un segundo ajuste.", "ε∞ plus the Kramers–Kronig transform of ε₂, including that tail. It is not a second fit.", "ε∞ 加上 ε₂ 的克拉默斯–克罗尼格变换，含那条尾部。不是第二次拟合。", "ε∞ mais a Kramers–Kronig de ε₂, incluindo essa cauda. Não é um segundo ajuste.");
    let g_nk = tx(lang, "índice de refracción.", "refractive index.", "折射率。", "índice de refração.");
    let g_k = tx(lang, "extinción, con el signo de ε₂, de modo que n² − k² = ε₁ y 2nk = ε₂.", "extinction, with the sign of ε₂, so that n² − k² = ε₁ and 2nk = ε₂.", "消光，符号与 ε₂ 相同，因此 n² − k² = ε₁ 且 2nk = ε₂。", "extinção, com o sinal de ε₂, de modo que n² − k² = ε₁ e 2nk = ε₂.");
    let g_r = tx(lang, "reflectividad de un medio semi-infinito a incidencia normal, calculada con esos n y k. No es la de una película sobre un sustrato.", "reflectivity of a semi-infinite medium at normal incidence, calculated from those n and k. It is not that of a film on a substrate.", "由这些 n 和 k 算出的半无限介质正入射反射率。不是衬底上薄膜的反射率。", "refletividade de um meio semi-infinito em incidência normal, calculada com esses n e k. Não é a de um filme sobre um substrato.");
    let g_loss = tx(lang, "función de pérdida: sus picos marcan los modos longitudinales y el plasmón.", "loss function: its peaks mark the longitudinal modes and the plasmon.", "损失函数：其峰标出纵模和等离激元。", "função de perda: seus picos marcam os modos longitudinais e o plásmon.");
    let g_peaks = tx(lang, "Máximos locales de ε₂ cuya prominencia pasa del 5 % del rango. Si ε₂ sigue subiendo al final de la ventana, ese máximo es el borde de absorción y no entra como pico cerrado.", "Local maxima of ε₂ whose prominence exceeds 5% of the range. If ε₂ is still rising at the end of the window, that maximum is the absorption edge and is not listed as a closed peak.", "突出度超过范围 5% 的 ε₂ 局部极大。若 ε₂ 在窗口末端仍在上升，该极大是吸收边，不算闭合峰。", "Máximos locais de ε₂ cuja proeminência passa de 5% da faixa. Se ε₂ continua subindo no fim da janela, esse máximo é a borda de absorção e não entra como pico fechado.");
    let g_charts = tx(lang, "Pasa el mouse para leer el valor exacto del punto más cercano de cada curva. Cada gráfica se baja en SVG o PNG con los botones de debajo.", "Move the pointer to read the exact value of the nearest point on each curve. Each chart downloads as SVG or PNG with the buttons below.", "移动指针可读取每条曲线最近点的精确值。每张图可用下方按钮下载为 SVG 或 PNG。", "Passe o ponteiro para ler o valor exato do ponto mais próximo de cada curva. Cada gráfico baixa em SVG ou PNG com os botões abaixo.");
    let data_word = tx(lang, "Datos", "Data", "数据", "Dados");
    let peaks_word = tx(lang, "Picos", "Peaks", "峰", "Picos");
    let charts_word = tx(lang, "Gráficas", "Charts", "图", "Gráficos");
    let result_word = tx(lang, "Resultado", "Result", "结果", "Resultado");
    format!(
        "<section class=\"panel results\"><div class=\"result-head\"><div><p class=\"eyebrow\">{result_word} · {label}</p><h2>{score}</h2><p class=\"muted\">{meta}</p><p>{status}</p></div>{downloads}</div>{warns}{facts}{table}{fit}{eps2}{eps1}{diff}{nk}{refl}{loss}{alpha}{sigma}{sigma2}{trans}{absorb}{ext}<ul class=\"glossary\"><li><strong>χ²</strong> {g_chi} {sym}. {g_chi2}</li><li><strong>χ²/dof</strong> {dof_help}</li><li><strong>±1σ</strong> {g_bar}</li><li><strong>{data_word}</strong> <span class=\"swatch swatch-data\"></span> {measure_note}</li><li><strong>ε₂</strong> <span class=\"swatch swatch-e2\"></span> {g_e2}</li><li><strong>ε₁</strong> <span class=\"swatch swatch-e1\"></span> {g_e1}</li><li><strong>n</strong> {g_nk} <strong>k</strong> {g_k} <strong>R</strong> {g_r} <strong>−Im(1/ε)</strong> {g_loss}</li>{units_note}<li><strong>{peaks_word}</strong> {g_peaks}</li><li><strong>{charts_word}</strong> {g_charts}</li></ul>{publish}{informe}</section>",
        label = escape(measure.symbol()),
        meta = escape(&format!(
            "{} {} · {} {} · {} · {}",
            report.points_used,
            tx(lang, "puntos", "points", "点", "pontos"),
            report.mfit,
            tx(lang, "coeficientes", "coefficients", "个系数", "coeficientes"),
            crate::i18n::unit_name(lang, unit.key()),
            source
        )),
        downloads = downloads(download_href, !report.peaks.is_empty(), lang),
        status = escape(&status),
        warns = warnings_markup(report),
        facts = facts(&items),
        table = peaks_table(&report.peaks, unit, "ε₂", lang),
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
        informe = report_button(download_href, lang),
        publish = crate::publish::form(download_href, report, lang),
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
