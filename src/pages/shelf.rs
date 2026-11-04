use lightmatter::chart::result_markup;
use lightmatter::i18n::{self, tx, Lang};
use lightmatter::measure::Measure;
use resuma::prelude::*;

use crate::shelf::{self, Card};
use crate::stats;

pub fn page(req: FlowRequest) -> View {
    let lang = crate::locale::current();
    set_page_title(tx(lang, "Estante de constantes ópticas | Lightmatter", "Shelf of optical constants | Lightmatter", "光学常数书架 | Lightmatter", "Estante de constantes ópticas | Lightmatter"));
    set_page_description(tx(
        lang,
        "Análisis publicados: ε, n, k y R recuperados con Kramers–Kronig. Cada ficha es un ajuste de alguien, con sus gráficas.",
        "Published analyses: ε, n, k, and R recovered with Kramers–Kronig. Each card is someone’s fit, with its charts.",
        "已发布的分析：用克拉默斯–克罗尼格得到的 ε、n、k 和 R。每张卡片是一个人的拟合和图。",
        "Análises publicadas: ε, n, k e R recuperados com Kramers–Kronig. Cada ficha é um ajuste de alguém, com os gráficos.",
    ));
    set_page_canonical(crate::seo::canonical("/shelf"));
    let q = req.query_param("q").unwrap_or("").trim().to_string();
    let measure = req.query_param("m").unwrap_or("").trim().to_string();
    let err = req.query_param("e").unwrap_or("");
    let cards = shelf::list();
    let shown: Vec<&Card> = cards
        .iter()
        .filter(|c| measure.is_empty() || c.measure == measure)
        .filter(|c| q.is_empty() || contains(&c.title, &q) || contains(&c.summary, &q))
        .collect();
    let notice = match err {
        "1" => view! { <p class="form-note" role="alert">{tx(lang, "No se pudo guardar. Prueba otra vez.", "It could not be saved. Try again.", "没能保存。请再试一次。", "Não foi possível guardar. Tente de novo.")}</p> },
        "2" => view! { <p class="form-note" role="alert">{tx(lang, "Demasiadas publicaciones seguidas. Espera unos minutos.", "Too many publications in a row. Wait a few minutes.", "连续发布太多。请等几分钟。", "Publicações demais seguidas. Espere alguns minutos.")}</p> },
        _ => view! { <p hidden=""></p> },
    };
    let filters = filters(lang, &cards, &measure, &q);
    let grid = if shown.is_empty() {
        view! {
            <p class="help">{tx(lang, "Todavía no hay un análisis con ese filtro. El primero se publica desde el laboratorio, al final del resultado.", "There is no analysis with that filter yet. The first one is published from the laboratory, at the end of a result.", "还没有符合这个筛选的分析。第一份从实验室的结果末尾发布。", "Ainda não há uma análise com esse filtro. A primeira se publica no laboratório, no fim do resultado.")}</p>
        }
    } else {
        View::fragment(shown.iter().map(|c| Child::View(card_view(c, lang))).collect())
    };
    let count = shown.len().to_string();
    let q_value = q.clone();
    view! {
        <main class="shelf-page">
            <section class="hero shelf-hero">
                <p class="eyebrow">{tx(lang, "Constantes que dejó alguien", "Constants someone left here", "别人留下的常数", "Constantes que alguém deixou")}</p>
                <h1>{tx(lang, "Estante", "Shelf", "书架", "Estante")}</h1>
                <p class="hero-lead">{tx(lang, "Un lugar para ver los análisis publicados: la curva medida, ε₁ y ε₂, n, k y R. Publicar el tuyo está al final de un resultado, con un nombre y un texto que puedes cambiar.", "A place to see published analyses: the measured curve, ε₁ and ε₂, n, k, and R. Publishing yours is at the end of a result, with a name and a text you can change.", "这里查看已发布的分析：测量曲线、ε₁ 和 ε₂、n、k 和 R。在结果末尾发布你的，名称和文字可以改。", "Um lugar para ver as análises publicadas: a curva medida, ε₁ e ε₂, n, k e R. Publicar a sua fica no fim de um resultado, com um nome e um texto que você pode mudar.")}</p>
            </section>
            {notice}
            <div class="shelf-layout">
                <aside class="panel shelf-side">
                    <h2>{tx(lang, "Medida", "Measurement", "测量", "Medida")}</h2>
                    {filters}
                </aside>
                <section>
                    <form class="shelf-search" method="get" action="/shelf">
                        <input type="hidden" name="m" value={measure.clone()} />
                        <label class="shelf-search-label">
                            {tx(lang, "Buscar", "Search", "搜索", "Buscar")}
                            <input name="q" type="search" value={q_value} placeholder={tx(lang, "nombre o una palabra del texto", "a name or a word from the text", "名称或文字中的词", "nome ou uma palavra do texto")} />
                        </label>
                        <button class="btn btn-ghost" type="submit">{tx(lang, "Buscar", "Search", "搜索", "Buscar")}</button>
                    </form>
                    <p class="muted shelf-count">{count}{tx(lang, " publicados", " published", " 份已发布", " publicados")}</p>
                    <div class="shelf-grid">{grid}</div>
                </section>
            </div>
        </main>
    }
}

pub fn one(req: FlowRequest) -> View {
    let lang = crate::locale::current();
    let id = req.param("id").unwrap_or("").to_string();
    let Some(opened) = shelf::load(&id) else {
        stage_response_status(404);
        set_page_title(tx(lang, "No está en el estante | Lightmatter", "Not on the shelf | Lightmatter", "书架上没有 | Lightmatter", "Não está no estante | Lightmatter"));
        set_page_robots("noindex, nofollow");
        return view! {
            <main class="panel">
                <h1>{tx(lang, "Esa ficha no está", "That card is not here", "没有这张卡片", "Essa ficha não está")}</h1>
                <p><a class="btn btn-primary" href="/shelf">{tx(lang, "Volver al estante", "Back to the shelf", "回到书架", "Voltar ao estante")}</a></p>
            </main>
        };
    };
    let title = opened.card.title.clone();
    set_page_title(&format!("{title} | Lightmatter"));
    set_page_description(&opened.card.summary);
    set_page_canonical(&crate::seo::canonical(&format!("/shelf/{id}")));
    let kind = i18n::measure_name(lang, &opened.card.measure).to_string();
    let when = stats::ago(opened.card.at, lang);
    let summary = opened.card.summary.clone();
    let peak = opened.card.peak.clone();
    let html = result_markup(&opened.source, &opened.report, &format!("/shelf/{id}/download"), lang);
    view! {
        <main class="shelf-one">
            <p class="eyebrow"><a href="/shelf">{tx(lang, "Estante", "Shelf", "书架", "Estante")}</a></p>
            <h1>{title}</h1>
            <p class="shelf-kicker">{kind}</p>
            <p class="hero-lead">{summary}</p>
            <p class="muted">{when}{" · "}{peak}</p>
            <div id="lm-live">{View::raw(html)}</div>
        </main>
    }
}

fn filters(lang: Lang, cards: &[Card], current: &str, q: &str) -> View {
    let mut rows = vec![filter_link(lang, "", tx(lang, "Todas", "All", "全部", "Todas"), current.is_empty(), q, cards.len())];
    for measure in Measure::ALL {
        let n = cards.iter().filter(|c| c.measure == measure.key()).count();
        rows.push(filter_link(lang, measure.key(), i18n::measure_name(lang, measure.key()), current == measure.key(), q, n));
    }
    View::fragment(rows)
}

fn filter_link(lang: Lang, key: &str, label: &str, on: bool, q: &str, n: usize) -> Child {
    let _ = lang;
    let href = if q.is_empty() {
        if key.is_empty() { "/shelf".to_string() } else { format!("/shelf?m={key}") }
    } else if key.is_empty() {
        format!("/shelf?q={}", urlencode(q))
    } else {
        format!("/shelf?m={key}&q={}", urlencode(q))
    };
    let name = label.to_string();
    let count = n.to_string();
    let class = if on { "is-current" } else { "" };
    Child::View(view! {
        <a href={href} class={class}>
            <span>{name}</span>
            <span class="mono">{count}</span>
        </a>
    })
}

fn card_view(card: &Card, lang: Lang) -> View {
    let href = format!("/shelf/{}", card.id);
    let title = card.title.clone();
    let summary = clip(&card.summary, 180);
    let kind = i18n::measure_name(lang, &card.measure).to_string();
    let when = stats::ago(card.at, lang);
    let meta = if card.peak.is_empty() { when } else { format!("{when} · {}", card.peak) };
    let spark = View::raw(shelf::spark_svg(&card.spark));
    view! {
        <a class="shelf-card" href={href}>
            {spark}
            <p class="shelf-kicker">{kind}</p>
            <h2>{title}</h2>
            <p>{summary}</p>
            <p class="muted">{meta}</p>
        </a>
    }
}

fn contains(hay: &str, needle: &str) -> bool {
    hay.to_lowercase().contains(&needle.to_lowercase())
}

fn clip(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{cut}…")
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
