use lightmatter::i18n::tx;
use resuma::prelude::*;

use crate::stats::{self, Dash, FitHit};

pub fn page(req: FlowRequest) -> View {
    let lang = crate::locale::current();
    set_page_robots("noindex, nofollow");
    if !crate::admin_auth::request_is_admin() {
        set_page_title("Lightmatter");
        set_page_description("Lightmatter");
        return login_view(req.query_param("e").unwrap_or(""), lang);
    }
    set_page_title(tx(lang, "Estadísticas | Lightmatter", "Statistics | Lightmatter", "统计 | Lightmatter", "Estatísticas | Lightmatter"));
    set_page_description(tx(lang, "Estadísticas privadas del laboratorio.", "Private statistics for the laboratory.", "实验室的私人统计。", "Estatísticas privadas do laboratório."));
    dash_view(&stats::dashboard(), lang)
}

fn login_view(err: &str, lang: lightmatter::i18n::Lang) -> View {
    let notice = match err {
        "1" => {
            let text = tx(lang, "Ese usuario o clave no coincide.", "That user or password does not match.", "用户或密码不对。", "Esse usuário ou senha não confere.");
            view! { <p class="form-note" role="alert">{text}</p> }
        }
        "2" => {
            let text = tx(lang, "Demasiados intentos. Espera unos minutos.", "Too many attempts. Wait a few minutes.", "尝试太多。请等几分钟。", "Tentativas demais. Espere alguns minutos.");
            view! { <p class="form-note" role="alert">{text}</p> }
        }
        _ => view! { <span hidden=""></span> },
    };
    let user = tx(lang, "Usuario", "User", "用户", "Usuário");
    let password = tx(lang, "Clave", "Password", "密码", "Senha");
    let enter = tx(lang, "Entrar", "Sign in", "进入", "Entrar");
    view! {
        <main class="admin-page">
            <section class="panel admin-login">
                <p class="eyebrow">"Lightmatter"</p>
                <h1>{enter}</h1>
                {notice}
                <form method="post" action="/admin/login" class="lab-form">
                    <label>
                        {user}
                        <input name="user" type="text" autocomplete="username" required=true />
                    </label>
                    <label>
                        {password}
                        <input name="password" type="password" autocomplete="current-password" required=true />
                    </label>
                    <button type="submit" class="btn btn-primary">{enter}</button>
                </form>
            </section>
        </main>
    }
}

fn dash_view(dash: &Dash, lang: lightmatter::i18n::Lang) -> View {
    let visits = stats::grouped(dash.home.saturating_add(dash.lab).saturating_add(dash.learn));
    let fits = stats::grouped(dash.fits);
    let errors = stats::grouped(dash.errors);
    let downloads = stats::grouped(dash.downloads);
    let home = stats::grouped(dash.home);
    let lab = stats::grouped(dash.lab);
    let learn = stats::grouped(dash.learn);
    view! {
        <main class="admin-page">
            <section class="panel">
                <div class="admin-head">
                    <div>
                        <p class="eyebrow">"Lightmatter"</p>
                        <h1>{tx(lang, "Estadísticas", "Statistics", "统计", "Estatísticas")}</h1>
                    </div>
                    <form method="post" action="/admin/logout">
                        <button type="submit" class="btn btn-ghost">{tx(lang, "Cerrar sesión", "Sign out", "退出", "Sair")}</button>
                    </form>
                </div>
                <div class="admin-grid">
                    {stat_card(tx(lang, "Visitas", "Visits", "访问", "Visitas"), &visits)}
                    {stat_card(tx(lang, "Ajustes", "Fits", "拟合", "Ajustes"), &fits)}
                    {stat_card(tx(lang, "Errores", "Errors", "错误", "Erros"), &errors)}
                    {stat_card(tx(lang, "Descargas", "Downloads", "下载", "Baixas"), &downloads)}
                    {stat_card(tx(lang, "Portada", "Home", "首页", "Início"), &home)}
                    {stat_card(tx(lang, "Laboratorio", "Laboratory", "实验室", "Laboratório"), &lab)}
                    {stat_card(tx(lang, "Guía", "Guide", "指南", "Guia"), &learn)}
                </div>
                <p class="help">{tx(lang, "Cuentan lo que pasa por este servidor y queda en su disco. Un ajuste sin red, dentro de la app instalada, no entra. No se guarda el espectro.", "They count what passes through this server and stays on its disk. An offline fit, inside the installed app, is not included. The spectrum is not stored.", "统计的是经过这台服务器并留在磁盘上的事。安装后的离线拟合不算。不保存光谱。", "Contam o que passa por este servidor e fica no disco. Um ajuste sem rede, dentro do app instalado, não entra. O espectro não é guardado.")}</p>
            </section>
            {count_table(tx(lang, "Por medida", "By measurement", "按测量", "Por medida"), &dash.measures, true, lang)}
            {count_table(tx(lang, "Descargas", "Downloads", "下载", "Baixas"), &dash.formats, false, lang)}
            {recent_table(&dash.recent, lang)}
        </main>
    }
}

fn stat_card(label: &'static str, value: &str) -> View {
    let value = value.to_string();
    view! {
        <article class="admin-stat">
            <p class="admin-stat-value">{value}</p>
            <p class="help">{label}</p>
        </article>
    }
}

fn count_table(title: &'static str, rows: &[(String, u64)], measures: bool, lang: lightmatter::i18n::Lang) -> View {
    if rows.is_empty() {
        let empty = tx(lang, "Todavía no hay nada que contar.", "Nothing to count yet.", "还没有可统计的内容。", "Ainda não há nada para contar.");
        return view! {
            <section class="panel admin-block">
                <h2>{title}</h2>
                <p class="help">{empty}</p>
            </section>
        };
    }
    let body = View::fragment(
        rows.iter()
            .map(|(key, n)| {
                let name = if measures {
                    if key == "lum" {
                        tx(lang, "Luminiscencia", "Luminescence", "发光", "Luminescência").to_string()
                    } else {
                        stats::measure_label(key)
                    }
                } else {
                    match key.as_str() {
                        "ajuste" => tx(lang, "Datos y ajuste", "Data and fit", "数据与拟合", "Dados e ajuste").into(),
                        "picos" => tx(lang, "Picos", "Peaks", "峰", "Picos").into(),
                        "informe" => tx(lang, "Informe", "Report", "报告", "Relatório").into(),
                        _ => stats::format_label(key),
                    }
                };
                let value = stats::grouped(*n);
                Child::View(view! {
                    <tr>
                        <td>{name}</td>
                        <td class="mono">{value}</td>
                    </tr>
                })
            })
            .collect(),
    );
    let head = if measures {
        tx(lang, "Medida", "Measurement", "测量", "Medida")
    } else {
        tx(lang, "Archivo", "File", "文件", "Arquivo")
    };
    let times = tx(lang, "Veces", "Times", "次数", "Vezes");
    view! {
        <section class="panel admin-block">
            <h2>{title}</h2>
            <div class="table-wrap">
                <table class="admin-table">
                    <thead>
                        <tr>
                            <th>{head}</th>
                            <th>{times}</th>
                        </tr>
                    </thead>
                    <tbody>{body}</tbody>
                </table>
            </div>
        </section>
    }
}

fn recent_table(rows: &[FitHit], lang: lightmatter::i18n::Lang) -> View {
    let title = tx(lang, "Ajustes recientes", "Recent fits", "最近的拟合", "Ajustes recentes");
    if rows.is_empty() {
        let empty = tx(lang, "Aparecen cuando alguien analiza un espectro en el laboratorio.", "They appear when someone fits a spectrum in the laboratory.", "有人在实验室分析光谱后会出现。", "Aparecem quando alguém analisa um espectro no laboratório.");
        return view! {
            <section class="panel admin-block">
                <h2>{title}</h2>
                <p class="help">{empty}</p>
            </section>
        };
    }
    let body = View::fragment(rows.iter().map(|row| Child::View(recent_row(row, lang))).collect());
    let when = tx(lang, "Cuándo", "When", "时间", "Quando");
    let file = tx(lang, "Archivo", "File", "文件", "Arquivo");
    let measure = tx(lang, "Medida", "Measurement", "测量", "Medida");
    let unit = tx(lang, "Unidad", "Unit", "单位", "Unidade");
    let points = tx(lang, "Puntos", "Points", "点数", "Pontos");
    let stop = tx(lang, "Parada", "Stop", "停止", "Parada");
    view! {
        <section class="panel admin-block">
            <h2>{title}</h2>
            <div class="table-wrap">
                <table class="admin-table">
                    <thead>
                        <tr>
                            <th>{when}</th>
                            <th>{file}</th>
                            <th>{measure}</th>
                            <th>{unit}</th>
                            <th>{points}</th>
                            <th>"χ²"</th>
                            <th>{stop}</th>
                        </tr>
                    </thead>
                    <tbody>{body}</tbody>
                </table>
            </div>
        </section>
    }
}

fn recent_row(row: &FitHit, lang: lightmatter::i18n::Lang) -> View {
    let when = stats::ago(row.at, lang);
    let source = row.source.clone();
    let measure = if row.measure == "lum" {
        tx(lang, "Luminiscencia", "Luminescence", "发光", "Luminescência").to_string()
    } else {
        stats::measure_label(&row.measure)
    };
    let unit = row.unit.clone();
    let points = stats::grouped(row.points as u64);
    let chisq = format_chisq(row.chisq);
    let stop = if row.converged {
        tx(lang, "cerca", "close", "接近", "perto")
    } else {
        tx(lang, "tope", "cap", "到顶", "teto")
    };
    view! {
        <tr>
            <td>{when}</td>
            <td>{source}</td>
            <td>{measure}</td>
            <td>{unit}</td>
            <td class="mono">{points}</td>
            <td class="mono">{chisq}</td>
            <td>{stop}</td>
        </tr>
    }
}

fn format_chisq(v: f64) -> String {
    if !v.is_finite() {
        return "—".into();
    }
    let a = v.abs();
    if a != 0.0 && (a >= 1.0e4 || a < 1.0e-3) {
        format!("{v:.3e}")
    } else {
        format!("{v:.4}")
    }
}
