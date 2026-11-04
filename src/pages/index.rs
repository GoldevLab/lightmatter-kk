use lightmatter::i18n::tx;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    let lang = crate::locale::current();
    crate::stats::record_visit("home");
    set_page_title(crate::seo::home_title(lang));
    set_page_description(crate::seo::home_description(lang));
    set_page_canonical(crate::seo::canonical("/"));
    set_page_json_ld(crate::seo::home_json_ld(lang));
    let faq: Vec<View> = crate::seo::faq(lang)
        .into_iter()
        .map(|(q, a)| {
            view! {
                <article>
                    <h3>{q}</h3>
                    <p>{a}</p>
                </article>
            }
        })
        .collect();
    let eyebrow = tx(lang, "Kuzmenko · KK discreto", "Kuzmenko · discrete KK", "Kuzmenko · 离散 KK", "Kuzmenko · KK discreta");
    let title = tx(lang, "Ajuste variacional Kramers–Kronig", "Variational Kramers–Kronig fit", "变分克拉默斯–克罗尼格拟合", "Ajuste variacional Kramers–Kronig");
    let lead = tx(
        lang,
        "Carga un espectro, ajusta ε₂ con una base triangular y recupera ε₁ por la transformada discreta. Instálala como aplicación: el mismo motor Rust sigue calculando sin conexión. Este Lightmatter es el laboratorio, no la empresa de chips fotónicos.",
        "Load a spectrum, fit ε₂ on a triangular basis, and recover ε₁ from the discrete transform. Install it as an app: the same Rust engine keeps calculating offline. This Lightmatter is the laboratory, not the photonic-chip company.",
        "载入光谱，用三角基拟合 ε₂，并由离散变换得到 ε₁。安装为应用后，同一个 Rust 引擎离线继续计算。这里的 Lightmatter 是实验室，不是光子芯片公司。",
        "Carregue um espectro, ajuste ε₂ com uma base triangular e recupere ε₁ pela transformada discreta. Instale como aplicativo: o mesmo motor Rust continua calculando sem rede. Este Lightmatter é o laboratório, não a empresa de chips fotônicos.",
    );
    let open_lab = tx(lang, "Abrir el laboratorio", "Open the laboratory", "打开实验室", "Abrir o laboratório");
    let learn = tx(lang, "Aprender el análisis", "Learn the analysis", "学习分析方法", "Aprender a análise");
    let shelf_btn = tx(lang, "Ver el estante", "Browse the shelf", "查看书架", "Ver o estante");
    let what = tx(lang, "Qué calcula", "What it calculates", "它计算什么", "O que calcula");
    let s1 = tx(lang, "Cargar el espectro", "Load the spectrum", "载入光谱", "Carregar o espectro");
    let s1p = tx(lang, "En el laboratorio suelta un archivo o pega ω y la medida. Sirve ε₂, ε₁, n, k, R, α, T o luminiscencia.", "In the laboratory, drop a file or paste ω and the measurement. ε₂, ε₁, n, k, R, α, T, or luminescence.", "在实验室放下文件，或粘贴 ω 和测量值。可用 ε₂、ε₁、n、k、R、α、T 或发光。", "No laboratório solte um arquivo ou cole ω e a medida. Serve ε₂, ε₁, n, k, R, α, T ou luminescência.");
    let s2 = tx(lang, "Ajustar con Kuzmenko", "Fit with Kuzmenko", "用 Kuzmenko 拟合", "Ajustar com Kuzmenko");
    let s2p = tx(lang, "Levenberg–Marquardt busca el ε₂ cuya Kramers–Kronig reproduce tu columna. ε₁ no se ajusta aparte.", "Levenberg–Marquardt looks for the ε₂ whose Kramers–Kronig transform reproduces your column. ε₁ is not fitted on its own.", "Levenberg–Marquardt 寻找其克拉默斯–克罗尼格变换能再现你那一列的 ε₂。ε₁ 不单独拟合。", "Levenberg–Marquardt procura o ε₂ cuja Kramers–Kronig reproduz a sua coluna. ε₁ não é ajustado à parte.");
    let s3 = tx(lang, "Leer las constantes", "Read the constants", "读取常数", "Ler as constantes");
    let s3p = tx(lang, "De ese par salen n, k, R, α, σ₁ y σ₂. Con espesor, T y la absorbancia. La guía está en Aprender.", "From that pair come n, k, R, α, σ₁, and σ₂. With a thickness, T and the absorbance. The guide is under Learn.", "由这对量得到 n、k、R、α、σ₁ 和 σ₂。有厚度时还有 T 和吸光度。指南在「学习」。", "Desse par saem n, k, R, α, σ₁ e σ₂. Com espessura, T e a absorbância. O guia está em Aprender.");
    let questions = tx(lang, "Preguntas", "Questions", "问题", "Perguntas");
    view! {
        <main class="home-page">
            <section class="hero">
                <p class="eyebrow">{eyebrow}</p>
                <h1>{title}</h1>
                <p class="hero-lead">{lead}</p>
                <div class="hero-actions">
                    <NavLink href="/lab" class="btn btn-primary">{open_lab}</NavLink>
                    <NavLink href="/shelf" class="btn btn-ghost">{shelf_btn}</NavLink>
                    <NavLink href="/learn" class="btn btn-ghost">{learn}</NavLink>
                </div>
            </section>
            <section class="tool-pick" aria-labelledby="method-title">
                <h2 id="method-title">{what}</h2>
                <ol class="howto-grid">
                    <li>
                        <h3>{s1}</h3>
                        <p>{s1p}</p>
                    </li>
                    <li>
                        <h3>{s2}</h3>
                        <p>{s2p}</p>
                    </li>
                    <li>
                        <h3>{s3}</h3>
                        <p>{s3p}</p>
                    </li>
                </ol>
            </section>
            <section class="panel" aria-labelledby="faq-title">
                <h2 id="faq-title">{questions}</h2>
                <div class="faq-list">{faq}</div>
            </section>
        </main>
    }
}
