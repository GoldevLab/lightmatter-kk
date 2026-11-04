use resuma::prelude::*;

use lightmatter::chart::result_markup;
use lightmatter::fit::FitReport;
use lightmatter::i18n::{self, tx, Lang};
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

fn form_state(report: &FitReport, lang: Lang) -> FormState {
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
            tx(
                lang,
                "El cuadro ya está en eV: cada nm pasó a E = 1239.84/λ. La unidad quedó en eV para no convertirlo otra vez. Si pegas el archivo original, elige nm antes de analizar.",
                "The box is already in eV: each nm became E = 1239.84/λ. The unit stayed on eV so it is not converted again. If you paste the original file, choose nm before fitting.",
                "文本框已经是 eV：每个 nm 已换成 E = 1239.84/λ。单位保持为 eV，避免再转换一次。若粘贴原始文件，分析前请选择 nm。",
                "A caixa já está em eV: cada nm passou a E = 1239.84/λ. A unidade ficou em eV para não converter de novo. Se colar o arquivo original, escolha nm antes de ajustar.",
            )
            .into(),
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
    let lang = crate::locale::current();
    crate::stats::record_visit("lab");
    set_page_title(crate::seo::lab_title(lang));
    set_page_description(crate::seo::lab_description(lang));
    let err = req.query_param("e").unwrap_or("").to_string();
    let report_id = req.query_param("r").unwrap_or("").to_string();
    if err.is_empty() && report_id.is_empty() {
        set_page_canonical(crate::seo::canonical("/lab"));
        set_page_json_ld(crate::seo::webpage_json_ld(
            crate::seo::lab_title(lang),
            crate::seo::lab_description(lang),
            "/lab",
            lang,
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
        Some(item) => form_state(&item.report, lang),
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
    let measures: Vec<(&str, &str)> = Measure::ALL.iter().map(|m| (m.key(), i18n::measure_name(lang, m.key()))).collect();
    let units: Vec<(&str, &str)> = Unit::ALL.iter().map(|u| (u.key(), i18n::unit_name(lang, u.key()))).collect();
    let measure_select = View::raw(select("measure", &measures, form.measure.key()));
    let unit_select = View::raw(select("unit", &units, form.unit.key()));
    let has_result = loaded.is_some() || !report_id.is_empty();
    let result = match loaded {
        Some(item) => {
            let html = result_markup(&item.source, &item.report, &format!("/lab/download/{report_id}"), lang);
            view! { <div id="lm-live">{View::raw(html)}</div> }
        }
        None if !report_id.is_empty() => view! { <div id="lm-live"><p class="form-error">{tx(lang, "Ese ajuste no está en este equipo.", "That fit is not on this device.", "这份拟合不在这台设备上。", "Esse ajuste não está neste aparelho.")}</p></div> },
        None => View::raw("<div id=\"lm-live\" class=\"is-idle\"></div>"),
    };
    let axis_note = if form.axis_note.is_empty() {
        view! { <p hidden=""></p> }
    } else {
        let note = form.axis_note.clone();
        view! { <p class="form-note" role="status">{note}</p> }
    };
    let page_class = if has_result { "lab-page has-result" } else { "lab-page" };
    let eyebrow = tx(lang, "Sin cuenta · se instala", "No account · installs", "无需账户 · 可安装", "Sem conta · instala");
    let h1 = tx(lang, "Laboratorio", "Laboratory", "实验室", "Laboratório");
    let lead = tx(lang, "Pega frecuencia y lo que mediste: ε₂, ε₁, n, k, R, α, T o luminiscencia. Con las ópticas, Levenberg–Marquardt busca el ε₂ cuyo par Kramers–Kronig reproduce tu medida; de ahí salen ε₁, n, k, R, la pérdida, α, σ₁, σ₂ y los picos. Con el espesor salen también T y la absorbancia. La luminiscencia no pasa por Kramers–Kronig: salen picos, anchos, áreas, centroide y la forma de la absorción.", "Paste the frequency and what you measured: ε₂, ε₁, n, k, R, α, T, or luminescence. For the optical ones, Levenberg–Marquardt looks for the ε₂ whose Kramers–Kronig pair reproduces your measurement; from that pair come ε₁, n, k, R, the loss, α, σ₁, σ₂, and the peaks. With a thickness, T and the absorbance too. Luminescence does not go through Kramers–Kronig: you get peaks, widths, areas, the centroid, and the absorption shape.", "粘贴频率和测量值：ε₂、ε₁、n、k、R、α、T 或发光。对光学量，Levenberg–Marquardt 寻找其克拉默斯–克罗尼格配对能再现测量的 ε₂；由此得到 ε₁、n、k、R、能量损失、α、σ₁、σ₂ 和峰。有厚度时还有 T 和吸光度。发光不经过克拉默斯–克罗尼格：得到峰、宽度、面积、质心和吸收线形。", "Cole a frequência e o que mediu: ε₂, ε₁, n, k, R, α, T ou luminescência. Nas ópticas, Levenberg–Marquardt procura o ε₂ cujo par Kramers–Kronig reproduz a medida; daí saem ε₁, n, k, R, a perda, α, σ₁, σ₂ e os picos. Com a espessura saem também T e a absorbância. A luminescência não passa por Kramers–Kronig: saem picos, larguras, áreas, centroide e a forma da absorção.");
    let legend1 = tx(lang, "1 · Espectro", "1 · Spectrum", "1 · 光谱", "1 · Espectro");
    let help1 = tx(lang, "Dos columnas: ω positiva y tu medida. La tercera, si la tienes, es el error de ese punto: pesa el χ² y dibuja la barra ±1σ. Una línea de texto, o que empiece por #, se ignora. Al soltar un archivo, sus números pasan al cuadro y la curva sale al lado, antes de pulsar analizar.", "Two columns: positive ω and your measurement. The third, if you have it, is that point’s error: it weights χ² and draws the ±1σ bar. A text line, or one that starts with #, is ignored. When you drop a file, its numbers go into the box and the curve appears beside it, before you press fit.", "两列：正的 ω 和测量值。若有第三列，是该点的误差：它给 χ² 加权并画出 ±1σ 误差棒。文字行或以 # 开头的行会被忽略。放下文件后，数字进入文本框，曲线立刻出现在旁边，不必先按分析。", "Duas colunas: ω positiva e a sua medida. A terceira, se existir, é o erro desse ponto: pesa o χ² e desenha a barra ±1σ. Uma linha de texto, ou que comece por #, é ignorada. Ao soltar um arquivo, os números passam para a caixa e a curva aparece ao lado, antes de analisar.");
    let what = tx(lang, "Qué mediste", "What you measured", "测量的是", "O que você mediu");
    let unit_of = tx(lang, "Unidad de ω", "Unit of ω", "ω 的单位", "Unidade de ω");
    let thick = tx(lang, "Espesor (µm)", "Thickness (µm)", "厚度 (µm)", "Espessura (µm)");
    let thick_ph = tx(lang, "para T y A", "for T and A", "用于 T 和 A", "para T e A");
    let temp = tx(lang, "T de la muestra (K)", "Sample T (K)", "样品温度 T (K)", "T da amostra (K)");
    let help2 = tx(lang, "R y T entre 0 y 1, o en %. α y T necesitan ω en cm⁻¹, eV o nm; ajustar T además pide el espesor. Si lo escribes en otra medida, el resultado incluye la transmitancia de una lámina en aire y la absorbancia. En nm el eje pasa a eV (E = 1239.84/λ). La temperatura solo entra en la luminiscencia.", "R and T between 0 and 1, or in %. α and T need ω in cm⁻¹, eV, or nm; fitting T also needs the thickness. If you enter it for another measurement, the result includes the transmittance of a slab in air and the absorbance. In nm the axis becomes eV (E = 1239.84/λ). Temperature is used only for luminescence.", "R 和 T 在 0 到 1 之间，或用百分数。α 和 T 需要 ω 为 cm⁻¹、eV 或 nm；拟合 T 还需要厚度。若在其他测量里填写厚度，结果会包含空气中薄片的透射率和吸光度。nm 会换成 eV（E = 1239.84/λ）。温度只用于发光。", "R e T entre 0 e 1, ou em %. α e T precisam de ω em cm⁻¹, eV ou nm; ajustar T também pede a espessura. Se você a escrever em outra medida, o resultado inclui a transmitância de uma lâmina no ar e a absorbância. Em nm o eixo passa a eV (E = 1239.84/λ). A temperatura só entra na luminescência.");
    let drop_title = tx(lang, "Suelta aquí un archivo", "Drop a file here", "把文件拖到这里", "Solte um arquivo aqui");
    let drop_name = tx(lang, ".dat, .txt o .csv · dos o tres columnas · hasta 4 MB", ".dat, .txt, or .csv · two or three columns · up to 4 MB", ".dat、.txt 或 .csv · 两列或三列 · 最大 4 MB", ".dat, .txt ou .csv · duas ou três colunas · até 4 MB");
    let drop_btn = tx(lang, "Elegir archivo", "Choose a file", "选择文件", "Escolher arquivo");
    let data_label = tx(lang, "Datos", "Data", "数据", "Dados");
    let graph = tx(lang, "Gráfica", "Plot", "图", "Gráfico");
    let preview_empty = tx(lang, "La curva de lo que cargues aparece aquí, sin esperar al ajuste.", "The curve of what you load appears here, before the fit.", "载入的曲线会出现在这里，不必等待拟合。", "A curva do que você carregar aparece aqui, sem esperar o ajuste.");
    let legend2 = tx(lang, "2 · Cómo ajustar", "2 · How to fit", "2 · 如何拟合", "2 · Como ajustar");
    let drude = tx(lang, " Fondo Drude", " Drude background", " Drude 背景", " Fundo de Drude");
    let log_mesh = tx(lang, " Eje ω en escala log", " ω axis on a log scale", " ω 轴用对数刻度", " Eixo ω em escala log");
    let extended = tx(lang, " Dibujar fuera del rango", " Draw outside the range", " 画出范围之外", " Desenhar fora do intervalo");
    let help3 = tx(lang, "Drude resta un fondo de electrones libres antes de ajustar ε₂. Todos los puntos entran en χ² (hasta 20000 filas). Con ε₂ cada frecuencia medida es una ancla, hasta 6000. Con R, T, n, k o α cada paso es denso y el modelo usa hasta 500 anclas, puestas en los picos y valles y repartidas en el resto. El eje ω pasa a log si el rango supera ×20. Fuera de tus datos, ε₂ baja en línea recta hasta cero y esa cola entra en ε₁.", "Drude subtracts a free-electron background before fitting ε₂. Every point enters χ² (up to 20000 rows). For ε₂ each measured frequency is an anchor, up to 6000. For R, T, n, k, or α each step is dense and the model uses up to 500 anchors, placed on the peaks and valleys and spread over the rest. The ω axis becomes log if the range exceeds ×20. Outside your data, ε₂ falls in a straight line to zero and that tail enters ε₁.", "Drude 在拟合 ε₂ 之前减去自由电子背景。所有点都进入 χ²（最多 20000 行）。对 ε₂，每个测量频率是一个锚点，最多 6000 个。对 R、T、n、k 或 α，每一步都很密，模型最多用 500 个锚点，放在峰和谷上，其余再铺开。若范围超过 ×20，ω 轴改为对数。数据之外，ε₂ 直线降到零，这条尾巴进入 ε₁。", "Drude subtrai um fundo de elétrons livres antes de ajustar ε₂. Todos os pontos entram no χ² (até 20000 linhas). Com ε₂ cada frequência medida é uma âncora, até 6000. Com R, T, n, k ou α cada passo é denso e o modelo usa até 500 âncoras, postas nos picos e vales e repartidas no resto. O eixo ω passa a log se o intervalo superar ×20. Fora dos dados, ε₂ desce em linha reta até zero e essa cauda entra em ε₁.");
    let eps_help = tx(lang, "ε₁ por encima de tu rango. Vacío: con n o ε₁ se ajusta; con R se estima del extremo alto. Con α, k o T no sale de la medida: vacío usa 1 y el índice se queda cerca de 1. Si conoces n donde la muestra es transparente, escribe n².", "ε₁ above your range. Blank: with n or ε₁ it is fitted; with R it is estimated from the high-frequency end. With α, k, or T it is not fixed by the measurement: blank uses 1 and the index stays near 1. If you know n where the sample is transparent, enter n².", "你的范围之上的 ε₁。留空：对 n 或 ε₁ 会拟合；对 R 从高频端估计。对 α、k 或 T，测量并不确定它：留空则用 1，折射率接近 1。若知道透明区的 n，填写 n²。", "ε₁ acima do seu intervalo. Vazio: com n ou ε₁ é ajustado; com R é estimado da ponta alta. Com α, k ou T não sai da medida: vazio usa 1 e o índice fica perto de 1. Se você conhece n onde a amostra é transparente, escreva n².");
    let wp_help = tx(lang, "Frecuencia de plasma del Drude.", "Drude plasma frequency.", "Drude 等离子体频率。", "Frequência de plasma de Drude.");
    let gamma_help = tx(lang, "Ancho de amortiguamiento.", "Damping width.", "阻尼宽度。", "Largura de amortecimento.");
    let steps = tx(lang, "Pasos", "Steps", "步数", "Passos");
    let steps_help = tx(lang, "Máximo. Sigue mientras un punto quede lejos. En un reststrahlen, pasar de 80 baja el χ² y rellena el valle; la punta se estanca.", "Maximum. It continues while a point is still far. On a reststrahlen, going past 80 lowers χ² and fills the valley; the tip stalls.", "上限。只要还有点离得远就继续。对剩余射线，超过 80 步会降低 χ² 并填平谷；尖峰会停住。", "Máximo. Continua enquanto um ponto estiver longe. Num reststrahlen, passar de 80 baixa o χ² e enche o vale; a ponta estaciona.");
    let smooth = tx(lang, "Suavidad", "Smoothness", "平滑", "Suavidade");
    let smooth_help = tx(lang, "Frena oscilaciones entre anclas interiores. No empuja los extremos hacia cero. 0 = solo los datos.", "Holds back oscillations between interior anchors. It does not push the ends toward zero. 0 = data only.", "抑制内部锚点之间的振荡。不把两端推向零。0 = 只用数据。", "Freia oscilações entre âncoras interiores. Não empurra as pontas para zero. 0 = só os dados.");
    let neg = tx(lang, "Si ε₂ baja de 0", "If ε₂ goes below 0", "若 ε₂ 低于 0", "Se ε₂ fica abaixo de 0");
    let neg_help = tx(lang, "Castiga ε₂ negativa. 0 = se permite. En un medio pasivo ε₂ es mayor o igual que 0.", "Penalizes negative ε₂. 0 = allowed. In a passive medium ε₂ is greater than or equal to 0.", "惩罚负的 ε₂。0 = 允许。无源介质中 ε₂ 大于或等于 0。", "Penaliza ε₂ negativa. 0 = permitida. Num meio passivo ε₂ é maior ou igual a 0.");
    let submit = tx(lang, "Analizar la medida", "Fit the measurement", "分析测量", "Analisar a medida");
    view! {
        <main class={page_class}>
            <section class="panel">
                <p class="eyebrow">{eyebrow}</p>
                <h1>{h1}</h1>
                <p class="hero-lead">{lead}</p>
                {err_view}
                <form id="ajuste" method="post" action="/lab/run" enctype="multipart/form-data" class="lab-form">
                    <fieldset>
                        <legend>{legend1}</legend>
                        <p class="help">{help1}</p>
                        <div class="measure-row">
                            <label>{what}{measure_select}</label>
                            <label>{unit_of}{unit_select}</label>
                            <label data-lm-unless="lum">{thick}<input name="thickness" type="text" inputmode="decimal" placeholder={thick_ph} value={form.thickness.clone()} /></label>
                            <label data-lm-for="lum">{temp}<input name="temperature" type="text" inputmode="decimal" placeholder="300" value={form.temperature.clone()} /></label>
                        </div>
                        {axis_note}
                        <p class="help">{help2}</p>
                        <label class="dropzone" data-lm-drop="">
                            <input class="file-input" name="file" type="file" accept=".dat,.txt,.csv,text/plain" />
                            <span class="drop-icon" aria-hidden="true">"↥"</span>
                            <span class="drop-text">
                                <span class="drop-title">{drop_title}</span>
                                <span class="drop-name" data-lm-file="">{drop_name}</span>
                            </span>
                            <span class="drop-btn">{drop_btn}</span>
                        </label>
                        <div class="spectrum-split">
                            <label class="grow">
                                {data_label}
                                <textarea name="spectrum" rows="14" spellcheck="false" placeholder={"1  0.20\n2  0.35\n3  0.50\n4  0.62\n5  0.55\n6  0.48\n7  0.40\n8  0.30"}>{form.spectrum.clone()}</textarea>
                            </label>
                            <figure class="chart preview-chart" data-lm-preview="">
                                <figcaption>{graph}</figcaption>
                                <div class="preview-slot" data-lm-preview-slot="">
                                    <p class="preview-empty">{preview_empty}</p>
                                </div>
                                <p class="preview-meta" data-lm-preview-meta=""></p>
                            </figure>
                        </div>
                    </fieldset>
                    <fieldset>
                        <legend>{legend2}</legend>
                        <div class="checks">
                            <label class="check">{drude_box}{drude}</label>
                            <label class="check">{log_box}{log_mesh}</label>
                            <label class="check">{ext_box}{extended}</label>
                        </div>
                        <p class="help">{help3}</p>
                        <div class="drude-fields">
                            <label>"ε∞"<input name="eps_inf" type="text" inputmode="decimal" placeholder="1" value={form.eps_inf.clone()} /><span class="help">{eps_help}</span></label>
                            <label>"ωₚ"<input name="omega_p" type="text" inputmode="decimal" placeholder="auto" value={form.omega_p.clone()} /><span class="help">{wp_help}</span></label>
                            <label>"γ"<input name="gamma" type="text" inputmode="decimal" placeholder="auto" value={form.gamma.clone()} /><span class="help">{gamma_help}</span></label>
                            <label>{steps}<input name="iter" type="number" value={form.iter.clone()} min="1" max="240" /><span class="help">{steps_help}</span></label>
                            <label>{smooth}<input name="smooth" type="number" value={form.smooth.clone()} min="0" max="1000000" step="any" /><span class="help">{smooth_help}</span></label>
                            <label>{neg}<input name="neg" type="number" value={form.neg.clone()} min="0" max="1000000" step="any" /><span class="help">{neg_help}</span></label>
                        </div>
                    </fieldset>
                    <button class="btn btn-primary" type="submit">{submit}</button>
                </form>
            </section>
            {result}
            <script type="module" src="/js/offline.js?v=13"></script>
        </main>
    }
}
