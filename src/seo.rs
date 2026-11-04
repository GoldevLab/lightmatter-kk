//! Origen público, canónicos y JSON-LD. `SITE_URL` pasa al dominio propio cuando exista.

use serde_json::{json, Value};

pub const HOME_TITLE: &str = "Kramers–Kronig variacional — ajuste de espectros ópticos | Lightmatter";
pub const HOME_DESCRIPTION: &str = "Ajusta ε₂ con triángulos y recupera ε₁, n, k, R, α y la conductividad con la Kramers–Kronig de Kuzmenko. Laboratorio gratis, sin cuenta. No es la empresa de chips fotónicos.";

pub const LAB_TITLE: &str = "Laboratorio Kramers–Kronig — ajusta tu espectro | Lightmatter";
pub const LAB_DESCRIPTION: &str = "Pega ε₂, ε₁, n, k, R, α, T o luminiscencia. Levenberg–Marquardt con la restricción de Kuzmenko. Sin cuenta: el cálculo es Rust en este equipo.";

pub const LEARN_TITLE: &str = "Guía del ajuste Kramers–Kronig de Kuzmenko | Lightmatter";
pub const LEARN_DESCRIPTION: &str = "Cómo funciona el laboratorio: la medida, el eje, ε₁, ε₂, n, k, R, α, σ₁, σ₂, T, la absorbancia y la luminiscencia. Fórmulas y límites del método.";

pub const FAQ: &[(&str, &str)] = &[
    (
        "¿Qué es Lightmatter?",
        "Un laboratorio web para ajustar espectros ópticos con la restricción variacional de Kramers–Kronig de Kuzmenko (2005). No es la empresa de computación fotónica que usa el mismo nombre.",
    ),
    (
        "¿Qué medidas acepta?",
        "ε₂, ε₁, n, k, reflectividad a incidencia normal, absorción α, transmitancia de una lámina y luminiscencia. De las ópticas salen el resto de constantes. La luminiscencia no pasa por Kramers–Kronig.",
    ),
    (
        "¿Hace falta una cuenta?",
        "No. El ajuste corre en este equipo, en Rust. Se puede instalar y sigue calculando sin red.",
    ),
    (
        "¿De dónde sale ε₁?",
        "De ε∞ más la transformada de Kramers–Kronig del ε₂ ajustado. No es un segundo ajuste independiente de la parte real.",
    ),
    (
        "¿La curva queda idéntica a los datos?",
        "Si mediste ε₂ y hay una ancla por punto, puede pasar por los datos. Con R, una raya reststrahlen de pocos cm⁻¹ puede quedarse corta: la banda ancha deja ε₁ positivo y la punta se estanca.",
    ),
];

/// Origen canónico. Con el dominio comprado, basta poner `SITE_URL=https://tu-dominio`.
pub fn origin() -> String {
    std::env::var("SITE_URL")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| s.starts_with("http://") || s.starts_with("https://"))
        .unwrap_or_else(|| "https://lightmatter-kk.fly.dev".into())
}

pub fn canonical(path: &str) -> String {
    let path = if path == "/" { "" } else { path };
    format!("{}{path}", origin())
}

pub fn home_json_ld() -> String {
    let origin = origin();
    let faq: Vec<Value> = FAQ
        .iter()
        .map(|(q, a)| {
            json!({
                "@type": "Question",
                "name": q,
                "acceptedAnswer": { "@type": "Answer", "text": a }
            })
        })
        .collect();
    let doc = json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "Organization",
                "@id": format!("{origin}/#org"),
                "name": "Lightmatter",
                "url": origin,
                "disambiguatingDescription": "Laboratorio web de ajuste de espectros ópticos con Kramers–Kronig. No es la empresa de computación fotónica del mismo nombre."
            },
            {
                "@type": "WebSite",
                "@id": format!("{origin}/#website"),
                "name": "Lightmatter",
                "url": origin,
                "inLanguage": "es",
                "publisher": { "@id": format!("{origin}/#org") }
            },
            {
                "@type": "WebApplication",
                "@id": format!("{origin}/#app"),
                "name": "Lightmatter",
                "alternateName": ["Laboratorio Kramers–Kronig", "ajuste variacional de Kuzmenko"],
                "url": format!("{origin}/lab"),
                "applicationCategory": "EducationalApplication",
                "operatingSystem": "Any",
                "browserRequirements": "Requires JavaScript",
                "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" },
                "featureList": [
                    "Ajuste variacional de ε₂ con base triangular",
                    "ε₁ por Kramers–Kronig discreta",
                    "n, k, R, α, conductividad, transmitancia y absorbancia",
                    "Luminiscencia: picos, centroide y forma de absorción",
                    "Funciona instalada y sin red"
                ],
                "description": HOME_DESCRIPTION
            },
            {
                "@type": "HowTo",
                "name": "Ajustar un espectro con Lightmatter",
                "description": "Cargar la medida, decir qué es y leer ε₁, ε₂ y las constantes que salen del par.",
                "totalTime": "PT2M",
                "step": [
                    {
                        "@type": "HowToStep",
                        "position": 1,
                        "name": "Cargar el espectro",
                        "text": "En /lab suelta un archivo o pega dos columnas: ω y lo que mediste. Elige la magnitud y la unidad."
                    },
                    {
                        "@type": "HowToStep",
                        "position": 2,
                        "name": "Ajustar",
                        "text": "Levenberg–Marquardt busca el ε₂ cuya Kramers–Kronig reproduce tu columna. ε∞ vacío se estima o se ajusta según la medida."
                    },
                    {
                        "@type": "HowToStep",
                        "position": 3,
                        "name": "Leer las constantes",
                        "text": "De ese par salen n, k, R, la pérdida, α, σ₁ y σ₂. Con espesor salen T y la absorbancia. La guía está en /learn."
                    }
                ]
            },
            {
                "@type": "FAQPage",
                "@id": format!("{origin}/#faq"),
                "mainEntity": faq
            }
        ]
    });
    serde_json::to_string(&doc).unwrap_or_else(|_| "{}".into())
}

pub fn webpage_json_ld(name: &str, description: &str, path: &str) -> String {
    let origin = origin();
    let doc = json!({
        "@context": "https://schema.org",
        "@type": "WebPage",
        "name": name,
        "description": description,
        "url": canonical(path),
        "inLanguage": "es",
        "isPartOf": { "@id": format!("{origin}/#website") }
    });
    serde_json::to_string(&doc).unwrap_or_else(|_| "{}".into())
}
