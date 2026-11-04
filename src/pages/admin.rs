use resuma::prelude::*;

use crate::stats::{self, Dash, FitHit};

pub fn page(req: FlowRequest) -> View {
    set_page_title("Admin | Lightmatter");
    set_page_description("Estadísticas privadas del laboratorio.");
    set_page_robots("noindex, nofollow");
    if !crate::admin_auth::request_is_admin() {
        return login_view(req.query_param("e").unwrap_or(""));
    }
    dash_view(&stats::dashboard())
}

fn login_view(err: &str) -> View {
    let notice = match err {
        "1" => view! {
            <p class="form-note" role="alert">"Ese usuario o clave no coincide."</p>
        },
        "2" => view! {
            <p class="form-note" role="alert">"Demasiados intentos. Espera unos minutos."</p>
        },
        _ => view! { <span hidden=""></span> },
    };
    view! {
        <main class="admin-page">
            <section class="panel admin-login">
                <p class="eyebrow">"Lightmatter"</p>
                <h1>"Admin"</h1>
                <p class="hero-lead">"Las visitas, los ajustes y las descargas de este laboratorio."</p>
                {notice}
                <form method="post" action="/admin/login" class="lab-form">
                    <label>
                        "Usuario"
                        <input name="user" type="text" autocomplete="username" required=true />
                    </label>
                    <label>
                        "Clave"
                        <input name="password" type="password" autocomplete="current-password" required=true />
                    </label>
                    <button type="submit" class="btn btn-primary">"Entrar"</button>
                </form>
            </section>
        </main>
    }
}

fn dash_view(dash: &Dash) -> View {
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
                        <h1>"Estadísticas"</h1>
                    </div>
                    <form method="post" action="/admin/logout">
                        <button type="submit" class="btn btn-ghost">"Cerrar sesión"</button>
                    </form>
                </div>
                <div class="admin-grid">
                    {stat_card("Visitas", &visits)}
                    {stat_card("Ajustes", &fits)}
                    {stat_card("Errores", &errors)}
                    {stat_card("Descargas", &downloads)}
                    {stat_card("Portada", &home)}
                    {stat_card("Laboratorio", &lab)}
                    {stat_card("Guía", &learn)}
                </div>
                <p class="help">"Cuentan lo que pasa por este servidor y queda en su disco. Un ajuste sin red, dentro de la app instalada, no entra. No se guarda el espectro."</p>
            </section>
            {count_table("Por medida", &dash.measures, true)}
            {count_table("Descargas", &dash.formats, false)}
            {recent_table(&dash.recent)}
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

fn count_table(title: &'static str, rows: &[(String, u64)], measures: bool) -> View {
    if rows.is_empty() {
        return view! {
            <section class="panel admin-block">
                <h2>{title}</h2>
                <p class="help">"Todavía no hay nada que contar."</p>
            </section>
        };
    }
    let body = View::fragment(
        rows.iter()
            .map(|(key, n)| {
                let name = if measures { stats::measure_label(key) } else { stats::format_label(key) };
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
    let head = if measures { "Medida" } else { "Archivo" };
    view! {
        <section class="panel admin-block">
            <h2>{title}</h2>
            <div class="table-wrap">
                <table class="admin-table">
                    <thead>
                        <tr>
                            <th>{head}</th>
                            <th>"Veces"</th>
                        </tr>
                    </thead>
                    <tbody>{body}</tbody>
                </table>
            </div>
        </section>
    }
}

fn recent_table(rows: &[FitHit]) -> View {
    if rows.is_empty() {
        return view! {
            <section class="panel admin-block">
                <h2>"Ajustes recientes"</h2>
                <p class="help">"Aparecen cuando alguien analiza un espectro en el laboratorio."</p>
            </section>
        };
    }
    let body = View::fragment(rows.iter().map(|row| Child::View(recent_row(row))).collect());
    view! {
        <section class="panel admin-block">
            <h2>"Ajustes recientes"</h2>
            <div class="table-wrap">
                <table class="admin-table">
                    <thead>
                        <tr>
                            <th>"Cuándo"</th>
                            <th>"Archivo"</th>
                            <th>"Medida"</th>
                            <th>"Unidad"</th>
                            <th>"Puntos"</th>
                            <th>"χ²"</th>
                            <th>"Parada"</th>
                        </tr>
                    </thead>
                    <tbody>{body}</tbody>
                </table>
            </div>
        </section>
    }
}

fn recent_row(row: &FitHit) -> View {
    let when = stats::ago(row.at);
    let source = row.source.clone();
    let measure = stats::measure_label(&row.measure);
    let unit = row.unit.clone();
    let points = stats::grouped(row.points as u64);
    let chisq = format_chisq(row.chisq);
    let stop = if row.converged { "cerca" } else { "tope" };
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
