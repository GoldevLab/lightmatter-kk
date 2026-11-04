//! Lightmatter — ajuste Kramers–Kronig en Resuma.

mod http;
mod pages;
mod store;

use std::net::SocketAddr;

use axum::routing::{get, post};
use pages::PagesRegistry;
use resuma::prelude::*;
use resuma::SeoKit;

fn view_transition_name(path: &str) -> String {
    let slug = path.trim_matches('/');
    if slug.is_empty() { "home".into() } else { slug.replace('/', "-") }
}

fn chrome(body: View) -> View {
    let path = current_request().map(|r| r.path).unwrap_or_else(|| "/".into());
    let vt = view_transition_name(&path);
    view! {
        <div class="app">
            <div class="liquid-orbs" aria-hidden="true">
                <div class="liquid-blob liquid-blob-a"></div>
                <div class="liquid-blob liquid-blob-b"></div>
                <div class="liquid-blob liquid-blob-c"></div>
            </div>
            <header class="site-header">
                <div class="header-inner">
                    <NavLink href="/" class="brand" activeClass="is-active" exact=true>
                        <span class="brand-mark" aria-hidden="true">"Lm"</span>
                        <span class="brand-name">"Lightmatter"</span>
                    </NavLink>
                    <span class="account">"PWA"</span>
                </div>
                <nav class="site-nav" aria-label="Secciones">
                    <NavLink href="/lab" class="site-nav-link" activeClass="is-active">"Laboratorio"</NavLink>
                    <NavLink href="/learn" class="site-nav-link" activeClass="is-active">"Aprender"</NavLink>
                </nav>
                <span class="nav-progress" aria-hidden="true"></span>
            </header>
            {with_view_transition(vt, vec![Child::View(body)])}
            <footer class="site-footer">
                <p>
                    <strong>"Lightmatter"</strong>
                    " — ajuste variacional de Kramers–Kronig (Kuzmenko, 2005). El cálculo es Rust. Se instala y sigue ajustando sin red."
                </p>
            </footer>
        </div>
    }
}

#[layout("/")]
fn RootLayout() -> View {
    chrome(view! { <Slot /> })
}

fn not_found() -> View {
    chrome(view! {
        <main class="panel">
            <h1>"No está esa ruta"</h1>
            <p class="hero-lead">"Lightmatter tiene la portada, el laboratorio y la guía."</p>
            <NavLink href="/" class="btn btn-primary">"Volver"</NavLink>
        </main>
    })
}

fn seo_kit() -> SeoKit {
    let origin = std::env::var("SITE_URL").unwrap_or_else(|_| "http://127.0.0.1:4321".into());
    let mut kit = SeoKit::new("Lightmatter", &origin)
        .with_locale("es_ES")
        .with_keywords("kramers kronig, kuzmenko, epsilon, espectroscopia, lightmatter")
        .with_llms_summary(
            "Lightmatter ajusta espectros con la restricción variacional de Kramers–Kronig de Kuzmenko. El núcleo numérico es Rust. No es una empresa de fotónica ni un simulador de precisión mixta.",
        );
    kit.theme_color = Some("#07141b".into());
    kit.author = "Lightmatter".into();
    kit
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let head = "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\" /><link rel=\"icon\" href=\"/icon.svg\" type=\"image/svg+xml\" />";
    const ICON: &[u8] = include_bytes!("icon.svg");
    let public = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
    let origin = std::env::var("SITE_URL").unwrap_or_else(|_| "http://127.0.0.1:4321".into());
    let mut serve = FlowServeOptions::default();
    if std::env::var_os("RESUMA_ADDR").is_none() {
        serve.addr = SocketAddr::from(([127, 0, 0, 1], 4321));
    }
    serve.security.csp.strict_dynamic = false;
    serve.security.csp.script_src.push("'wasm-unsafe-eval'".into());
    serve.security.csp.worker_src = vec!["'self'".into()];

    FlowApp::new()
        .with_title("Lightmatter — ajuste variacional Kramers–Kronig")
        .with_description("Ajuste de espectros con la restricción de Kramers–Kronig. Cálculo en Rust.")
        .with_site_url(origin)
        .with_head(head)
        .with_seo_kit(seo_kit())
        .with_html_theme(
            HtmlTheme::new(["spectrometer"])
                .dark(["spectrometer"])
                .cookie("lightmatter_theme")
                .storage_key("lightmatter-theme"),
        )
        .with_stylesheet("/images/lightmatter.css?v=8")
        .static_asset("/icon.svg", ICON, "image/svg+xml")
        .with_public_dir(public)
        .with_pwa(FlowPwaConfig {
            name: "Lightmatter".into(),
            short_name: "Lightmatter".into(),
            description: "Ajuste variacional Kramers–Kronig. Funciona instalada, sin red.".into(),
            theme_color: "#07141b".into(),
            background_color: "#07141b".into(),
            start_url: "/lab".into(),
            scope: "/".into(),
            cache_version: "lm-17".into(),
            display: "standalone".into(),
            orientation: "any".into(),
            lang: "es".into(),
            icon_char: Some("L".into()),
            precache_paths: vec!["/lab".into(), "/learn".into(), "/images/lightmatter.css?v=8".into(), "/js/offline.js?v=7".into()],
            shortcuts: vec![PwaShortcut {
                name: "Laboratorio".into(),
                short_name: "Lab".into(),
                url: "/lab".into(),
            }],
            offline_title: "Lightmatter está en este aparato".into(),
            offline_message: "Abre Laboratorio. El ajuste variacional corre en Rust dentro de la app, sin red.".into(),
            manifest_icons: Vec::new(),
        })
        .route("/lab/run", post(http::run_lab))
        .route("/lab/download/{id}", get(http::download))
        .route("/lab/download/{id}/{format}", get(http::download_as))
        .not_found(not_found)
        .auto_pages(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
            PagesRegistry,
        )
        .serve(serve)
        .await
}
