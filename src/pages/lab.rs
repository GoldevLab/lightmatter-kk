use resuma::prelude::*;

use lightmatter::chart::result_markup;
use lightmatter::fit::FitReport;
use lightmatter::measure::{Measure, Unit};

use crate::store;

struct FormState {
    spectrum: String,
    drude: bool,
    eps_inf: String,
    omega_p: String,
    gamma: String,
    log_mesh: bool,
    extended: bool,
    iter: String,
    smooth: String,
    neg: String,
    measure: Measure,
    unit: Unit,
    thickness: String,
    temperature: String,
    /// El cuadro ya no está en la unidad del archivo.
    axis_note: String,
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            spectrum: String::new(),
            drude: false,
            eps_inf: String::new(),
            omega_p: String::new(),
            gamma: String::new(),
            log_mesh: false,
            extended: false,
            iter: "20".into(),
            smooth: "0.08".into(),
            neg: "100".into(),
            measure: Measure::Eps2,
            unit: Unit::Other,
            thickness: String::new(),
            temperature: "300".into(),
            axis_note: String::new(),
        }
    }
}

fn form_state(report: &FitReport) -> FormState {
    let spectrum = if report.errors_given && report.data_sigma.len() == report.data_x.len() {
        report
            .data_x
            .iter()
            .zip(report.data_y.iter())
            .zip(report.data_sigma.iter())
            .map(|((x, y), s)| format!("{x:.6}  {y:.6}  {s:.6}"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        report
            .data_x
            .iter()
            .zip(report.data_y.iter())
            .map(|(x, y)| format!("{x:.6}  {y:.6}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let (drude, omega_p, gamma) = match &report.drude {
        Some(d) => (true, num(d.omega_p), num(d.gamma)),
        None => (false, String::new(), String::new()),
    };
    // Si el Drude usó un ε∞ (escrito o el 1 de vacío), el formulario lo conserva.
    // Si se estimó o se ajustó, el campo sigue vacío para no fijarlo en el siguiente envío.
    let eps_inf = report
        .options
        .eps_inf
        .or_else(|| report.drude.as_ref().map(|d| d.eps_inf))
        .map(num)
        .unwrap_or_default();
    let o = &report.options;
    // nm ya se pasó a eV dentro del ajuste. Si el cuadro vuelve a decir nm, el siguiente
    // análisis convierte esos eV otra vez.
    let (unit, axis_note) = if o.unit == Unit::Nm {
        (
            Unit::Ev,
            "El cuadro ya está en eV: cada nm pasó a E = 1239.84/λ. La unidad quedó en eV para no convertirlo otra vez. Si pegas el archivo original, elige nm antes de analizar.".into(),
        )
    } else {
        (o.unit, String::new())
    };
    FormState {
        spectrum,
        drude,
        eps_inf,
        omega_p,
        gamma,
        log_mesh: report.options.log_mesh,
        extended: report.options.extended,
        iter: report.options.iter_max.to_string(),
        smooth: num(report.options.smoothness),
        neg: num(report.options.negativity),
        measure: o.measure,
        unit,
        thickness: if o.thickness_um > 0.0 { num(o.thickness_um) } else { String::new() },
        temperature: num(o.temperature_k),
        axis_note,
    }
}

/// `selected="false"` también selecciona: el atributo solo va en la opción elegida.
fn select(name: &str, options: &[(&str, &str)], current: &str) -> String {
    let mut out = format!("<select name=\"{name}\">");
    for (key, label) in options {
        let sel = if *key == current { " selected" } else { "" };
        out.push_str(&format!("<option value=\"{key}\"{sel}>{label}</option>"));
    }
    out.push_str("</select>");
    out
}

fn num(v: f64) -> String {
    let s = format!("{v:.8}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

pub fn page(req: FlowRequest) -> View {
    crate::stats::record_visit("lab");
    set_page_title(crate::seo::LAB_TITLE);
    set_page_description(crate::seo::LAB_DESCRIPTION);
    let err = req.query_param("e").unwrap_or("").to_string();
    let report_id = req.query_param("r").unwrap_or("").to_string();
    if err.is_empty() && report_id.is_empty() {
        set_page_canonical(crate::seo::canonical("/lab"));
        set_page_json_ld(crate::seo::webpage_json_ld(
            crate::seo::LAB_TITLE,
            crate::seo::LAB_DESCRIPTION,
            "/lab",
        ));
    } else {
        set_page_robots("noindex, follow");
    }
    let loaded = if report_id.is_empty() {
        None
    } else {
        store::load_report(&report_id)
    };
    let err_view = if err.is_empty() {
        view! { <p hidden=""></p> }
    } else {
        view! { <p class="form-error" role="alert">{err}</p> }
    };
    let form = match &loaded {
        Some(item) => form_state(&item.report),
        None => FormState::default(),
    };
    // `checked="false"` el navegador lo trata como marcado. El atributo solo va si es verdad.
    let drude_box = if form.drude {
        view! { <input name="drude" type="checkbox" value="1" checked="" /> }
    } else {
        view! { <input name="drude" type="checkbox" value="1" /> }
    };
    let log_box = if form.log_mesh {
        view! { <input name="log_mesh" type="checkbox" value="1" checked="" /> }
    } else {
        view! { <input name="log_mesh" type="checkbox" value="1" /> }
    };
    let ext_box = if form.extended {
        view! { <input name="extended" type="checkbox" value="1" checked="" /> }
    } else {
        view! { <input name="extended" type="checkbox" value="1" /> }
    };
    let measures: Vec<(&str, &str)> = Measure::ALL.iter().map(|m| (m.key(), m.label())).collect();
    let units: Vec<(&str, &str)> = Unit::ALL.iter().map(|u| (u.key(), u.label())).collect();
    let measure_select = View::raw(select("measure", &measures, form.measure.key()));
    let unit_select = View::raw(select("unit", &units, form.unit.key()));
    let has_result = loaded.is_some() || !report_id.is_empty();
    let result = match loaded {
        Some(item) => {
            let html = result_markup(&item.source, &item.report, &format!("/lab/download/{report_id}"));
            view! { <div id="lm-live">{View::raw(html)}</div> }
        }
        None if !report_id.is_empty() => view! { <div id="lm-live"><p class="form-error">"Ese ajuste no está en este equipo."</p></div> },
        None => View::raw("<div id=\"lm-live\" class=\"is-idle\"></div>"),
    };
    let axis_note = if form.axis_note.is_empty() {
        view! { <p hidden=""></p> }
    } else {
        let note = form.axis_note.clone();
        view! { <p class="form-note" role="status">{note}</p> }
    };
    let page_class = if has_result { "lab-page has-result" } else { "lab-page" };
    view! {
        <main class={page_class}>
            <section class="panel">
                <p class="eyebrow">"Sin cuenta · se instala"</p>
                <h1>"Laboratorio"</h1>
                <p class="hero-lead">
                    "Pega frecuencia y lo que mediste: ε₂, ε₁, n, k, R, α, T o luminiscencia. Con las ópticas, Levenberg–Marquardt busca el ε₂ cuyo par Kramers–Kronig reproduce tu medida; de ahí salen ε₁, n, k, R, la pérdida, α, σ₁, σ₂ y los picos. Con el espesor salen también T y la absorbancia. La luminiscencia no pasa por Kramers–Kronig: salen picos, anchos, áreas, centroide y la forma de la absorción."
                </p>
                {err_view}
                <form id="ajuste" method="post" action="/lab/run" enctype="multipart/form-data" class="lab-form">
                    <fieldset>
                        <legend>"1 · Espectro"</legend>
                        <p class="help">"Dos columnas: ω positiva y tu medida. La tercera, si la tienes, es el error de ese punto: pesa el χ² y dibuja la barra ±1σ. Una línea de texto, o que empiece por #, se ignora. Al soltar un archivo, sus números pasan al cuadro y la curva sale al lado, antes de pulsar analizar."</p>
                        <div class="measure-row">
                            <label>"Qué mediste"{measure_select}</label>
                            <label>"Unidad de ω"{unit_select}</label>
                            <label data-lm-unless="lum">"Espesor (µm)"<input name="thickness" type="text" inputmode="decimal" placeholder="para T y A" value={form.thickness.clone()} /></label>
                            <label data-lm-for="lum">"T de la muestra (K)"<input name="temperature" type="text" inputmode="decimal" placeholder="300" value={form.temperature.clone()} /></label>
                        </div>
                        {axis_note}
                        <p class="help">"R y T entre 0 y 1, o en %. α y T necesitan ω en cm⁻¹, eV o nm; ajustar T además pide el espesor. Si lo escribes en otra medida, el resultado incluye la transmitancia de una lámina en aire y la absorbancia. En nm el eje pasa a eV (E = 1239.84/λ). La temperatura solo entra en la luminiscencia."</p>
                        <label class="dropzone" data-lm-drop="">
                            <input class="file-input" name="file" type="file" accept=".dat,.txt,.csv,text/plain" />
                            <span class="drop-icon" aria-hidden="true">"↥"</span>
                            <span class="drop-text">
                                <span class="drop-title">"Suelta aquí un archivo"</span>
                                <span class="drop-name" data-lm-file="">".dat, .txt o .csv · dos o tres columnas · hasta 4 MB"</span>
                            </span>
                            <span class="drop-btn">"Elegir archivo"</span>
                        </label>
                        <div class="spectrum-split">
                            <label class="grow">
                                "Datos"
                                <textarea name="spectrum" rows="14" spellcheck="false" placeholder={"1  0.20\n2  0.35\n3  0.50\n4  0.62\n5  0.55\n6  0.48\n7  0.40\n8  0.30"}>{form.spectrum.clone()}</textarea>
                            </label>
                            <figure class="chart preview-chart" data-lm-preview="">
                                <figcaption>"Gráfica"</figcaption>
                                <div class="preview-slot" data-lm-preview-slot="">
                                    <p class="preview-empty">"La curva de lo que cargues aparece aquí, sin esperar al ajuste."</p>
                                </div>
                                <p class="preview-meta" data-lm-preview-meta=""></p>
                            </figure>
                        </div>
                    </fieldset>
                    <fieldset>
                        <legend>"2 · Cómo ajustar"</legend>
                        <div class="checks">
                            <label class="check">{drude_box}" Fondo Drude"</label>
                            <label class="check">{log_box}" Eje ω en escala log"</label>
                            <label class="check">{ext_box}" Dibujar fuera del rango"</label>
                        </div>
                        <p class="help">"Drude resta un fondo de electrones libres antes de ajustar ε₂. Todos los puntos entran en χ² (hasta 20000 filas). Con ε₂ cada frecuencia medida es una ancla, hasta 6000. Con R, T, n, k o α cada paso es denso y el modelo usa hasta 500 anclas, puestas en los picos y valles y repartidas en el resto. El eje ω pasa a log si el rango supera ×20. Fuera de tus datos, ε₂ baja en línea recta hasta cero y esa cola entra en ε₁."</p>
                        <div class="drude-fields">
                            <label>"ε∞"<input name="eps_inf" type="text" inputmode="decimal" placeholder="1" value={form.eps_inf.clone()} /><span class="help">"ε₁ por encima de tu rango. Vacío: con n o ε₁ se ajusta; con R se estima del extremo alto. Con α, k o T no sale de la medida: vacío usa 1 y el índice se queda cerca de 1. Si conoces n donde la muestra es transparente, escribe n²."</span></label>
                            <label>"ωₚ"<input name="omega_p" type="text" inputmode="decimal" placeholder="auto" value={form.omega_p.clone()} /><span class="help">"Frecuencia de plasma del Drude."</span></label>
                            <label>"γ"<input name="gamma" type="text" inputmode="decimal" placeholder="auto" value={form.gamma.clone()} /><span class="help">"Ancho de amortiguamiento."</span></label>
                            <label>"Pasos"<input name="iter" type="number" value={form.iter.clone()} min="1" max="240" /><span class="help">"Máximo. Sigue mientras un punto quede lejos. En un reststrahlen, pasar de 80 baja el χ² y rellena el valle; la punta se estanca."</span></label>
                            <label>"Suavidad"<input name="smooth" type="number" value={form.smooth.clone()} min="0" max="1000000" step="any" /><span class="help">"Frena oscilaciones entre anclas interiores. No empuja los extremos hacia cero. 0 = solo los datos."</span></label>
                            <label>"Si ε₂ baja de 0"<input name="neg" type="number" value={form.neg.clone()} min="0" max="1000000" step="any" /><span class="help">"Castiga ε₂ negativa. 0 = se permite. En un medio pasivo ε₂ es mayor o igual que 0."</span></label>
                        </div>
                    </fieldset>
                    <button class="btn btn-primary" type="submit">"Analizar la medida"</button>
                </form>
            </section>
            {result}
            <script type="module" src="/js/offline.js?v=10"></script>
        </main>
    }
}
