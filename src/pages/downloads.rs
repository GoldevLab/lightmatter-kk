use lightmatter::i18n::tx;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    let lang = crate::locale::current();
    crate::stats::record_visit("downloads");
    set_page_title(tx(lang, "Descargas · Lightmatter", "Downloads · Lightmatter", "下载 · Lightmatter", "Downloads · Lightmatter"));
    set_page_description(tx(
        lang,
        "Instala Lightmatter como PWA o descarga la versión portable de Windows para usarla sin internet.",
        "Install Lightmatter as a PWA or download the Windows portable build to use it offline.",
        "安装 Lightmatter 作为 PWA，或下载 Windows 便携版以离线使用。",
        "Instale o Lightmatter como PWA ou baixe a versão portátil para Windows para usá-la offline.",
    ));
    set_page_canonical(crate::seo::canonical("/downloads"));

    let eyebrow = tx(lang, "Descargas", "Downloads", "下载", "Downloads");
    let title = tx(lang, "Instala Lightmatter o descarga la versión portable", "Install Lightmatter or download the portable build", "安装 Lightmatter 或下载便携版", "Instale o Lightmatter ou baixe a versão portátil");
    let lead = tx(
        lang,
        "Elige la opción que te sirva: guardar la app como PWA en tu navegador o bajar el paquete de Windows para usarlo sin internet.",
        "Choose the option that fits you: install the app as a PWA in your browser or download the Windows package for offline use.",
        "选择适合你的方式：在浏览器中安装 PWA，或下载 Windows 便携包以离线使用。",
        "Escolha a opção que te serve: instalar o app como PWA no navegador ou baixar o pacote para Windows para usá-lo offline.",
    );
    let install = tx(lang, "Instalar la app", "Install the app", "安装应用", "Instalar o app");
    let install_hint = tx(
        lang,
        "La app se guarda en tu navegador y queda disponible como aplicación en el sistema.",
        "The app is saved in your browser and becomes available as a standalone app on your system.",
        "该应用会保存到浏览器中，并在系统中作为独立应用可用。",
        "O app é salvo no navegador e fica disponível como aplicativo independente no sistema.",
    );
    let windows_desktop = tx(lang, "Descargar app de escritorio (Windows)", "Download Windows desktop app", "下载 Windows 桌面应用", "Baixar app de desktop (Windows)");
    let windows_desktop_note = tx(
        lang,
        "Instalador de Windows para la versión completa de escritorio.",
        "Windows installer for the full desktop app.",
        "完整版桌面应用的 Windows 安装程序。",
        "Instalador para Windows da versão completa do app.",
    );
    let portable_windows = tx(lang, "Descargar versión portable (Windows)", "Download portable Windows build", "下载 Windows 便携版", "Baixar versão portátil (Windows)");
    let portable_note = tx(
        lang,
        "Paquete listo para extraer y usar sin depender del navegador ni de internet.",
        "Ready-to-extract package for use without relying on the browser or internet.",
        "可直接解压并使用，不依赖浏览器或网络。",
        "Pacote pronto para extrair e usar sem depender do navegador nem da internet.",
    );

    let desktop_url = "https://github.com/GoldevLab/lightmatter-kk/releases/latest/download/lightmatter-setup.exe";
    let portable_url = "https://github.com/GoldevLab/lightmatter-kk/releases/latest/download/lightmatter-portable.zip";

    view! {
        <main class="panel download-page">
            <p class="eyebrow">{eyebrow}</p>
            <h1>{title}</h1>
            <p class="hero-lead">{lead}</p>
            <div class="download-grid">
                <article class="download-card">
                    <h2>{install}</h2>
                    <p>{install_hint}</p>
                    <button type="button" class="install-app btn btn-primary">{install}</button>
                    <p class="help" data-install-hint="" role="status" aria-live="polite" hidden=""></p>
                </article>

                <article class="download-card">
                    <h2>{windows_desktop}</h2>
                    <p>{windows_desktop_note}</p>
                    <a class="btn btn-primary" href={desktop_url} target="_blank" rel="noopener noreferrer">{windows_desktop}</a>
                </article>

                <article class="download-card">
                    <h2>{portable_windows}</h2>
                    <p>{portable_note}</p>
                    <a class="btn btn-ghost" href={portable_url} target="_blank" rel="noopener noreferrer">{portable_windows}</a>
                </article>
            </div>
        </main>
    }
}
