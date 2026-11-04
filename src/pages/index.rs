use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    crate::stats::record_visit("home");
    set_page_title(crate::seo::HOME_TITLE);
    set_page_description(crate::seo::HOME_DESCRIPTION);
    set_page_canonical(crate::seo::canonical("/"));
    set_page_json_ld(crate::seo::home_json_ld());
    let faq: Vec<View> = crate::seo::FAQ
        .iter()
        .map(|&(q, a)| {
            view! {
                <article>
                    <h3>{q}</h3>
                    <p>{a}</p>
                </article>
            }
        })
        .collect();
    view! {
        <main class="home-page">
            <section class="hero">
                <p class="eyebrow">"Kuzmenko · KK discreto"</p>
                <h1>"Ajuste variacional Kramers–Kronig"</h1>
                <p class="hero-lead">
                    "Carga un espectro, ajusta ε₂ con una base triangular y recupera ε₁ por la transformada discreta. Instálala como aplicación: el mismo motor Rust sigue calculando sin conexión. Este Lightmatter es el laboratorio, no la empresa de chips fotónicos."
                </p>
                <div class="hero-actions">
                    <NavLink href="/lab" class="btn btn-primary">"Abrir el laboratorio"</NavLink>
                    <NavLink href="/learn" class="btn btn-ghost">"Aprender el análisis"</NavLink>
                </div>
            </section>
            <section class="tool-pick" aria-labelledby="method-title">
                <h2 id="method-title">"Qué calcula"</h2>
                <ol class="howto-grid">
                    <li>
                        <h3>"Cargar el espectro"</h3>
                        <p>"En el laboratorio suelta un archivo o pega ω y la medida. Sirve ε₂, ε₁, n, k, R, α, T o luminiscencia."</p>
                    </li>
                    <li>
                        <h3>"Ajustar con Kuzmenko"</h3>
                        <p>"Levenberg–Marquardt busca el ε₂ cuya Kramers–Kronig reproduce tu columna. ε₁ no se ajusta aparte."</p>
                    </li>
                    <li>
                        <h3>"Leer las constantes"</h3>
                        <p>"De ese par salen n, k, R, α, σ₁ y σ₂. Con espesor, T y la absorbancia. La guía está en Aprender."</p>
                    </li>
                </ol>
            </section>
            <section class="panel" aria-labelledby="faq-title">
                <h2 id="faq-title">"Preguntas"</h2>
                <div class="faq-list">{faq}</div>
            </section>
        </main>
    }
}
