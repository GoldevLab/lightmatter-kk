use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Lightmatter — ajuste variacional Kramers–Kronig");
    set_page_description(
        "Ajuste de espectros con la restricción de Kramers–Kronig de Kuzmenko. El cálculo corre en Rust.",
    );
    view! {
        <main class="home-page">
            <section class="hero">
                <p class="eyebrow">"Kuzmenko · KK discreto"</p>
                <h1>"Ajuste variacional Kramers–Kronig"</h1>
                <p class="hero-lead">
                    "Carga un espectro, ajusta ε₂ con una base triangular y recupera ε₁ por la transformada discreta. Instálala como aplicación: el mismo motor Rust sigue calculando sin conexión."
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
                        <h3>"ε₂ variacional"</h3>
                        <p>"Coeficientes sobre triángulos en ω. Levenberg–Marquardt, suavidad y penalización si ε₂ baja de cero."</p>
                    </li>
                    <li>
                        <h3>"ε₁ por DKKT"</h3>
                        <p>"La parte real sale de la transformada de Kramers–Kronig discreta, no de un ajuste independiente."</p>
                    </li>
                    <li>
                        <h3>"Óptica derivada"</h3>
                        <p>"A partir de ε₁ y ε₂: n, índice de refracción; k, extinción; R, reflectividad a incidencia normal."</p>
                    </li>
                </ol>
            </section>
        </main>
    }
}
