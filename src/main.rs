//! Lightmatter — ajuste Kramers–Kronig en Resuma.

mod admin_auth;
mod http;
mod locale;
mod pages;
mod seo;
mod shelf;
mod stats;
mod store;

use std::net::SocketAddr;

use axum::routing::{get, post};
use lightmatter::i18n::tx;
use pages::PagesRegistry;
use resuma::prelude::*;
use resuma::SeoKit;

fn view_transition_name(path: &str) -> String {
    let slug = path.trim_matches('/');
    if slug.is_empty() { "home".into() } else { slug.replace('/', "-") }
}

fn chrome(body: View) -> View {
    let lang = locale::current();
    set_page_lang(lang.html());
    let path = current_request().map(|r| r.path).unwrap_or_else(|| "/".into());
    let nav_aria = tx(lang, "Secciones", "Sections", "栏目", "Seções");
    let home = tx(lang, "Inicio", "Home", "首页", "Início");
    let lab = tx(lang, "Laboratorio", "Laboratory", "实验室", "Laboratório");
    let learn = tx(lang, "Aprender", "Learn", "学习", "Aprender");
    let shelf_nav = tx(lang, "Estante", "Shelf", "书架", "Estante");
    let foot_lab = tx(lang, "Laboratorio Kramers–Kronig", "Kramers–Kronig laboratory", "克拉默斯–克罗尼格实验室", "Laboratório Kramers–Kronig");
    let foot_guide = tx(lang, "Guía del análisis", "Guide to the analysis", "分析指南", "Guia da análise");
    let foot_method = tx(lang, "El método", "The method", "方法", "O método");
    let foot_const = tx(lang, "Constantes ópticas", "Optical constants", "光学常数", "Constantes ópticas");
    let foot_lum = tx(lang, "Luminiscencia", "Luminescence", "发光", "Luminescência");
    let blurb = tx(
        lang,
        " — ajuste variacional de Kramers–Kronig (Kuzmenko, 2005). El cálculo es Rust. Se instala y sigue ajustando sin red. No es la empresa de chips fotónicos del mismo nombre.",
        " — variational Kramers–Kronig fit (Kuzmenko, 2005). The calculation is Rust. Install it and it keeps fitting offline. It is not the photonic-computing company of the same name.",
        " — 变分克拉默斯–克罗尼格拟合（Kuzmenko，2005）。计算用 Rust。安装后离线仍可拟合。不是同名的光子计算公司。",
        " — ajuste variacional de Kramers–Kronig (Kuzmenko, 2005). O cálculo é Rust. Instalado, continua ajustando sem rede. Não é a empresa de computação fotônica do mesmo nome.",
    );
    let install = tx(lang, "Instalar", "Install", "安装", "Instalar");
    let tools = nav_tools(lang, install);
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
                        <span class="brand-mark"><img src="/images/lightmatter-logo.jpg" alt="" width="256" height="256" /></span>
                        <span class="brand-name">"Lightmatter"</span>
                    </NavLink>
                    <nav class="site-nav" aria-label={nav_aria}>
                        <NavLink href="/lab" class="site-nav-link" activeClass="is-active">{lab}</NavLink>
                        <NavLink href="/shelf" class="site-nav-link" activeClass="is-active">{shelf_nav}</NavLink>
                        <NavLink href="/learn" class="site-nav-link" activeClass="is-active">{learn}</NavLink>
                        {tools}
                    </nav>
                </div>
                <span class="nav-progress" aria-hidden="true"></span>
            </header>
            {with_view_transition(vt, vec![Child::View(body)])}
            <footer class="site-footer">
                <nav class="seo-links" aria-label="Lightmatter">
                    <NavLink href="/">{home}</NavLink>
                    <span aria-hidden="true">" · "</span>
                    <NavLink href="/lab">{foot_lab}</NavLink>
                    <span aria-hidden="true">" · "</span>
                    <NavLink href="/learn">{foot_guide}</NavLink>
                    <span aria-hidden="true">" · "</span>
                    <a href="/learn#metodo">{foot_method}</a>
                    <span aria-hidden="true">" · "</span>
                    <a href="/learn#constantes">{foot_const}</a>
                    <span aria-hidden="true">" · "</span>
                    <a href="/learn#luminiscencia">{foot_lum}</a>
                </nav>
                <p>
                    <strong>"Lightmatter"</strong>
                    {blurb}
                </p>
            </footer>
        </div>
    }
}

fn nav_tools(lang: lightmatter::i18n::Lang, install: &'static str) -> View {
    let theme_aria = tx(lang, "Tema", "Theme", "主题", "Tema");
    let day = tx(lang, "Claro", "Light", "浅色", "Claro");
    let night = tx(lang, "Oscuro", "Dark", "深色", "Escuro");
    view! {
        <div class="nav-tools">
            <div class="theme-pair" role="group" aria-label={theme_aria}>
                <ThemeSwitch id="day">{day}</ThemeSwitch>
                <ThemeSwitch id="night">{night}</ThemeSwitch>
            </div>
            <button type="button" class="install-app" hidden="">{install}</button>
            <script src="/js/lang.js?v=4" defer=true></script>
        </div>
    }
}

#[layout("/")]
fn RootLayout() -> View {
    chrome(view! { <Slot /> })
}

fn not_found() -> View {
    let lang = locale::current();
    let title = tx(lang, "No está esa ruta", "That page is not here", "没有这个页面", "Essa página não está aqui");
    let lead = tx(
        lang,
        "Lightmatter tiene la portada, el laboratorio y la guía.",
        "Lightmatter has the home page, the laboratory, and the guide.",
        "Lightmatter 有首页、实验室和指南。",
        "Lightmatter tem a página inicial, o laboratório e o guia.",
    );
    let back = tx(lang, "Volver", "Back", "返回", "Voltar");
    chrome(view! {
        <main class="panel">
            <h1>{title}</h1>
            <p class="hero-lead">{lead}</p>
            <NavLink href="/" class="btn btn-primary">{back}</NavLink>
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
            "Portada /. Laboratorio /lab (ahí se pega el espectro). Estante /shelf, cada ficha en /shelf/id. Guía /learn, con anclas #dato #medidas #metodo #ajuste #constantes #lectura #luminiscencia #limites. \
             Un resultado concreto vive en /lab?r= y no se indexa. /lab/run y /lab/download son el cálculo y la descarga, no páginas.",
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
    let head = "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\" /><link rel=\"icon\" href=\"/images/lightmatter-logo.jpg\" type=\"image/jpeg\" /><link rel=\"apple-touch-icon\" href=\"/images/lightmatter-logo.jpg\" /><script type=\"module\" src=\"/js/offline.js?v=15\"></script>";
    const ICON: &[u8] = include_bytes!("icon.svg");
    let public = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
    let origin = seo::origin();
    let security_txt = format!(
        "Canonical: {origin}/.well-known/security.txt\nPreferred-Languages: es, en, zh, pt\nExpires: 2027-12-31T23:59:59Z\n"
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
        .with_title(seo::home_title(lightmatter::i18n::Lang::Es))
        .with_description(seo::home_description(lightmatter::i18n::Lang::Es))
        .with_site_url(origin)
        .with_head(head)
        .with_seo_kit(seo_kit())
        .with_html_theme(
            HtmlTheme::new(["day", "night"])
                .dark(["night"])
                .cookie("lightmatter_theme")
                .storage_key("lightmatter-theme"),
        )
        .with_stylesheet("/images/lightmatter.css?v=25")
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
            cache_version: "lm-41".into(),
            display: "standalone".into(),
            orientation: "any".into(),
            lang: "es".into(),
            icon_char: Some("L".into()),
            precache_paths: vec![
                "/".into(),
                "/lab".into(),
                "/learn".into(),
                "/images/lightmatter.css?v=25".into(),
                "/js/lang.js?v=4".into(),
                "/_resuma/ui.css?v=4679afe0d6db98bc".into(),
                "/_resuma/loader.js?v=2.0.1".into(),
                "/images/lightmatter-logo.jpg".into(),
                "/images/learn-kk.jpg".into(),
                "/images/learn-data.jpg".into(),
                "/images/learn-triangles.jpg".into(),
                "/images/learn-fit.jpg".into(),
                "/images/learn-constants.jpg".into(),
                "/js/offline.js?v=15".into(),
                "/robots.txt".into(),
                "/llms.txt".into(),
                "/sitemap.xml".into(),
            ],
            shortcuts: vec![PwaShortcut {
                name: "Laboratorio".into(),
                short_name: "Lab".into(),
                url: "/lab".into(),
            }],
            offline_title: "Lightmatter está en este aparato".into(),
            offline_message: "Abre Laboratorio. El ajuste variacional corre en Rust dentro de la app, sin red.".into(),
            manifest_icons: Vec::new(),
        })
        .route(
            "/lab/run",
            post(http::run_lab).layer(axum::extract::DefaultBodyLimit::max(http::MAX_REQUEST_BODY)),
        )
        .route("/lab/download/{id}", get(http::download))
        .route("/lab/download/{id}/{format}", get(http::download_as))
        .route("/shelf/publish", post(shelf::publish_form))
        .route("/shelf/{id}/download", get(shelf::download))
        .route("/shelf/{id}/download/{format}", get(shelf::download_as))
        .route("/lang/{code}", get(locale::set_lang))
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
