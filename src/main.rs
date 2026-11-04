//! Lightmatter — ajuste Kramers–Kronig en Resuma.

mod admin_auth;
mod http;
mod pages;
mod seo;
mod stats;
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
    let admin_link = if admin_auth::request_is_admin() {
        view! { <NavLink href="/admin" class="site-nav-link" activeClass="is-active">"Admin"</NavLink> }
    } else {
        view! { <span hidden=""></span> }
    };
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
                    <nav class="site-nav" aria-label="Secciones">
                        <NavLink href="/" class="site-nav-link" activeClass="is-active" exact=true>"Inicio"</NavLink>
                        <NavLink href="/lab" class="site-nav-link" activeClass="is-active">"Laboratorio"</NavLink>
                        <NavLink href="/learn" class="site-nav-link" activeClass="is-active">"Aprender"</NavLink>
                    {admin_link}
                    </nav>
                </div>
                <span class="nav-progress" aria-hidden="true"></span>
            </header>
            {with_view_transition(vt, vec![Child::View(body)])}
            <footer class="site-footer">
                <nav class="seo-links" aria-label="Lightmatter">
                    <NavLink href="/">"Inicio"</NavLink>
                    <span aria-hidden="true">" · "</span>
                    <NavLink href="/lab">"Laboratorio Kramers–Kronig"</NavLink>
                    <span aria-hidden="true">" · "</span>
                    <NavLink href="/learn">"Guía del análisis"</NavLink>
                    <span aria-hidden="true">" · "</span>
                    <a href="/learn#metodo">"El método"</a>
                    <span aria-hidden="true">" · "</span>
                    <a href="/learn#constantes">"Constantes ópticas"</a>
                    <span aria-hidden="true">" · "</span>
                    <a href="/learn#luminiscencia">"Luminiscencia"</a>
                </nav>
                <p>
                    <strong>"Lightmatter"</strong>
                    " — ajuste variacional de Kramers–Kronig (Kuzmenko, 2005). El cálculo es Rust. Se instala y sigue ajustando sin red. No es la empresa de chips fotónicos del mismo nombre."
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
    let origin = seo::origin();
    let mut kit = SeoKit::new("Lightmatter", &origin)
        .with_locale("es_ES")
        .with_keywords(
            "kramers-kronig, kramers kronig variacional, kuzmenko, función dieléctrica, \
             epsilon, reflectividad, índice de refracción, espectroscopia óptica, \
             ajuste de espectros, dielectric function, optical constants, reflectivity fit, \
             lightmatter laboratorio",
        )
        .with_llms_summary(
            "Lightmatter es un laboratorio gratis para ajustar espectros ópticos con la restricción variacional de Kramers–Kronig de Kuzmenko (2005). \
             El cálculo es Rust, en el equipo, sin cuenta. Acepta ε₂, ε₁, n, k, R, α, T y luminiscencia. \
             No es Lightmatter Inc., la empresa de computación fotónica, ni un simulador de chips.",
        )
        .with_llms_section(
            "Páginas",
            "Portada /. Laboratorio /lab (ahí se pega el espectro). Guía /learn, con anclas #dato #eje #medidas #metodo #ajuste #constantes #lectura #luminiscencia #limites. \
             Un resultado concreto vive en /lab?r= y no se indexa. /lab/run y /lab/download son el cálculo y la descarga, no páginas. /admin es privado.",
        )
        .with_llms_section(
            "Cómo se ajusta",
            "ε₂ es una suma de triángulos. ε₁ es ε∞ más la Kramers–Kronig de esos triángulos (el kernel g resta el término central). \
             Levenberg–Marquardt compara la magnitud medida, no ε₂, salvo que hayas medido ε₂. La luminiscencia no usa Kramers–Kronig.",
        )
        .with_llms_section(
            "In English",
            "Lightmatter is a free variational Kramers–Kronig lab (Kuzmenko 2005) for optical spectra: dielectric function, refractive index, extinction, normal-incidence reflectivity, absorption, slab transmittance, and luminescence. \
             It is not the photonic-computing company of the same name. Live origin is the Canonical line below; a custom domain will replace SITE_URL.",
        );
    kit.theme_color = Some("#07141b".into());
    kit.author = "Lightmatter".into();
    kit.ai.disallow = vec!["/lab/run".into(), "/lab/download".into(), "/admin".into()];
    kit
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let head = "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\" /><link rel=\"icon\" href=\"/icon.svg\" type=\"image/svg+xml\" />";
    const ICON: &[u8] = include_bytes!("icon.svg");
    let public = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
    let origin = seo::origin();
    let security_txt = format!(
        "Canonical: {origin}/.well-known/security.txt\nPreferred-Languages: es, en\nExpires: 2027-12-31T23:59:59Z\n"
    );
    let security_txt: &'static [u8] = Box::leak(security_txt.into_bytes().into_boxed_slice());
    let mut serve = FlowServeOptions::default();
    if std::env::var_os("RESUMA_ADDR").is_none() {
        serve.addr = SocketAddr::from(([127, 0, 0, 1], 4321));
    }
    serve.security.csp.strict_dynamic = false;
    serve.security.csp.script_src.push("'wasm-unsafe-eval'".into());
    serve.security.csp.worker_src = vec!["'self'".into()];

    FlowApp::new()
        .with_title(seo::HOME_TITLE)
        .with_description(seo::HOME_DESCRIPTION)
        .with_site_url(origin)
        .with_head(head)
        .with_seo_kit(seo_kit())
        .with_html_theme(
            HtmlTheme::new(["spectrometer"])
                .dark(["spectrometer"])
                .cookie("lightmatter_theme")
                .storage_key("lightmatter-theme"),
        )
        .with_stylesheet("/images/lightmatter.css?v=14")
        .with_sitemap_exclude(["/admin"])
        .static_asset("/icon.svg", ICON, "image/svg+xml")
        .static_asset("/.well-known/security.txt", security_txt, "text/plain; charset=utf-8")
        .with_public_dir(public)
        .with_pwa(FlowPwaConfig {
            name: "Lightmatter".into(),
            short_name: "Lightmatter".into(),
            description: "Ajuste variacional Kramers–Kronig. Funciona instalada, sin red.".into(),
            theme_color: "#07141b".into(),
            background_color: "#07141b".into(),
            start_url: "/lab".into(),
            scope: "/".into(),
            cache_version: "lm-27".into(),
            display: "standalone".into(),
            orientation: "any".into(),
            lang: "es".into(),
            icon_char: Some("L".into()),
            precache_paths: vec!["/".into(), "/lab".into(), "/learn".into(), "/images/lightmatter.css?v=14".into(), "/js/offline.js?v=10".into(), "/robots.txt".into(), "/llms.txt".into(), "/sitemap.xml".into()],
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
        .route("/admin/login", post(admin_auth::login))
        .route("/admin/logout", post(admin_auth::logout))
        .not_found(not_found)
        .auto_pages(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
            PagesRegistry,
        )
        .serve(serve)
        .await
}
