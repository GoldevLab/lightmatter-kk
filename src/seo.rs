//! Origen público, canónicos y JSON-LD. `SITE_URL` pasa al dominio propio cuando exista.

use lightmatter::i18n::{tx, Lang};
use serde_json::{json, Value};

pub fn home_title(lang: Lang) -> &'static str {
    tx(
        lang,
        "Kramers–Kronig variacional — ajuste de espectros ópticos | Lightmatter",
        "Variational Kramers–Kronig — fit optical spectra | Lightmatter",
        "变分克拉默斯–克罗尼格 — 拟合光学光谱 | Lightmatter",
        "Kramers–Kronig variacional — ajuste de espectros ópticos | Lightmatter",
    )
}

pub fn home_description(lang: Lang) -> &'static str {
    tx(
        lang,
        "Ajusta ε₂ con triángulos y recupera ε₁, n, k, R, α y la conductividad con la Kramers–Kronig de Kuzmenko. Laboratorio gratis, sin cuenta. No es la empresa de chips fotónicos.",
        "Fit ε₂ with triangles and recover ε₁, n, k, R, α, and the conductivity with Kuzmenko’s Kramers–Kronig. Free lab, no account. Not the photonic-chip company.",
        "用三角基拟合 ε₂，并用 Kuzmenko 的克拉默斯–克罗尼格关系得到 ε₁、n、k、R、α 和电导率。免费实验室，无需账户。不是光子芯片公司。",
        "Ajusta ε₂ com triângulos e recupera ε₁, n, k, R, α e a condutividade com a Kramers–Kronig de Kuzmenko. Laboratório grátis, sem conta. Não é a empresa de chips fotônicos.",
    )
}

pub fn lab_title(lang: Lang) -> &'static str {
    tx(
        lang,
        "Laboratorio Kramers–Kronig — ajusta tu espectro | Lightmatter",
        "Kramers–Kronig laboratory — fit your spectrum | Lightmatter",
        "克拉默斯–克罗尼格实验室 — 拟合你的光谱 | Lightmatter",
        "Laboratório Kramers–Kronig — ajuste o seu espectro | Lightmatter",
    )
}

pub fn lab_description(lang: Lang) -> &'static str {
    tx(
        lang,
        "Pega ε₂, ε₁, n, k, R, α, T o luminiscencia. Levenberg–Marquardt con la restricción de Kuzmenko. Sin cuenta: el cálculo es Rust en este equipo.",
        "Paste ε₂, ε₁, n, k, R, α, T, or luminescence. Levenberg–Marquardt with Kuzmenko’s constraint. No account: the calculation is Rust on this device.",
        "粘贴 ε₂、ε₁、n、k、R、α、T 或发光光谱。用 Kuzmenko 约束做 Levenberg–Marquardt。无需账户：计算是这台设备上的 Rust。",
        "Cole ε₂, ε₁, n, k, R, α, T ou luminescência. Levenberg–Marquardt com a restrição de Kuzmenko. Sem conta: o cálculo é Rust neste aparelho.",
    )
}

pub fn learn_title(lang: Lang) -> &'static str {
    tx(
        lang,
        "Guía del ajuste Kramers–Kronig de Kuzmenko | Lightmatter",
        "Guide to Kuzmenko’s Kramers–Kronig fit | Lightmatter",
        "Kuzmenko 克拉默斯–克罗尼格拟合指南 | Lightmatter",
        "Guia do ajuste Kramers–Kronig de Kuzmenko | Lightmatter",
    )
}

pub fn learn_description(lang: Lang) -> &'static str {
    tx(
        lang,
        "Cómo funciona el laboratorio: la medida, el eje, ε₁, ε₂, n, k, R, α, σ₁, σ₂, T, la absorbancia y la luminiscencia. Fórmulas y límites del método.",
        "How the lab works: the measurement, the axis, ε₁, ε₂, n, k, R, α, σ₁, σ₂, T, absorbance, and luminescence. Formulas and limits of the method.",
        "实验室如何工作：测量、坐标轴、ε₁、ε₂、n、k、R、α、σ₁、σ₂、T、吸光度和发光。公式与方法的界限。",
        "Como o laboratório funciona: a medida, o eixo, ε₁, ε₂, n, k, R, α, σ₁, σ₂, T, a absorbância e a luminescência. Fórmulas e limites do método.",
    )
}

pub fn faq(lang: Lang) -> [(&'static str, &'static str); 5] {
    [
        (
            tx(lang, "¿Qué es Lightmatter?", "What is Lightmatter?", "Lightmatter 是什么？", "O que é o Lightmatter?"),
            tx(
                lang,
                "Un laboratorio web para ajustar espectros ópticos con la restricción variacional de Kramers–Kronig de Kuzmenko (2005). No es la empresa de computación fotónica que usa el mismo nombre.",
                "A web lab that fits optical spectra with Kuzmenko’s variational Kramers–Kronig constraint (2005). It is not the photonic-computing company that uses the same name.",
                "一个用 Kuzmenko（2005）变分克拉默斯–克罗尼格约束拟合光学光谱的网络实验室。不是使用同一名称的光子计算公司。",
                "Um laboratório web para ajustar espectros ópticos com a restrição variacional de Kramers–Kronig de Kuzmenko (2005). Não é a empresa de computação fotônica que usa o mesmo nome.",
            ),
        ),
        (
            tx(lang, "¿Qué medidas acepta?", "Which measurements does it take?", "接受哪些测量？", "Quais medidas ele aceita?"),
            tx(
                lang,
                "ε₂, ε₁, n, k, reflectividad a incidencia normal, absorción α, transmitancia de una lámina y luminiscencia. De las ópticas salen el resto de constantes. La luminiscencia no pasa por Kramers–Kronig.",
                "ε₂, ε₁, n, k, normal-incidence reflectivity, absorption α, slab transmittance, and luminescence. The other constants come from the optical ones. Luminescence does not go through Kramers–Kronig.",
                "ε₂、ε₁、n、k、正入射反射率、吸收系数 α、薄片透射率和发光。其余常数由光学量得到。发光不经过克拉默斯–克罗尼格。",
                "ε₂, ε₁, n, k, refletividade em incidência normal, absorção α, transmitância de uma lâmina e luminescência. As outras constantes saem das ópticas. A luminescência não passa por Kramers–Kronig.",
            ),
        ),
        (
            tx(lang, "¿Hace falta una cuenta?", "Do I need an account?", "需要账户吗？", "Precisa de uma conta?"),
            tx(
                lang,
                "No. El ajuste corre en este equipo, en Rust. Se puede instalar y sigue calculando sin red.",
                "No. The fit runs on this device, in Rust. You can install it and it keeps calculating offline.",
                "不需要。拟合在这台设备上用 Rust 运行。可以安装，离线继续计算。",
                "Não. O ajuste roda neste aparelho, em Rust. Pode ser instalado e continua calculando sem rede.",
            ),
        ),
        (
            tx(lang, "¿De dónde sale ε₁?", "Where does ε₁ come from?", "ε₁ 从哪里来？", "De onde sai ε₁?"),
            tx(
                lang,
                "De ε∞ más la transformada de Kramers–Kronig del ε₂ ajustado. No es un segundo ajuste independiente de la parte real.",
                "From ε∞ plus the Kramers–Kronig transform of the fitted ε₂. It is not a second, independent fit of the real part.",
                "来自 ε∞ 加上已拟合 ε₂ 的克拉默斯–克罗尼格变换。不是对实部另做一次独立拟合。",
                "De ε∞ mais a transformada de Kramers–Kronig do ε₂ ajustado. Não é um segundo ajuste independente da parte real.",
            ),
        ),
        (
            tx(lang, "¿La curva queda idéntica a los datos?", "Does the curve match the data exactly?", "曲线会和数据完全重合吗？", "A curva fica idêntica aos dados?"),
            tx(
                lang,
                "Si mediste ε₂ y hay una ancla por punto, puede pasar por los datos. Con R, una raya reststrahlen de pocos cm⁻¹ puede quedarse corta: la banda ancha deja ε₁ positivo y la punta se estanca.",
                "If you measured ε₂ and there is one anchor per point, it can pass through the data. For R, a reststrahlen line a few cm⁻¹ wide can fall short: the broad band keeps ε₁ positive and the tip stalls.",
                "若测量的是 ε₂ 且每个点有一个锚点，曲线可以穿过数据。对 R，几 cm⁻¹ 宽的剩余射线可能拟合不足：宽带来使 ε₁ 为正，尖峰会停住。",
                "Se você mediu ε₂ e há uma âncora por ponto, ela pode passar pelos dados. Com R, uma linha reststrahlen de poucos cm⁻¹ pode ficar curta: a banda larga deixa ε₁ positivo e a ponta estaciona.",
            ),
        ),
    ]
}

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

pub fn home_json_ld(lang: Lang) -> String {
    let origin = origin();
    let faq: Vec<Value> = faq(lang)
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
                "disambiguatingDescription": tx(lang,
                    "Laboratorio web de ajuste de espectros ópticos con Kramers–Kronig. No es la empresa de computación fotónica del mismo nombre.",
                    "Web lab for Kramers–Kronig fits of optical spectra. Not the photonic-computing company of the same name.",
                    "用克拉默斯–克罗尼格拟合光学光谱的网络实验室。不是同名的光子计算公司。",
                    "Laboratório web de ajuste de espectros ópticos com Kramers–Kronig. Não é a empresa de computação fotônica do mesmo nome.")
            },
            {
                "@type": "WebSite",
                "@id": format!("{origin}/#website"),
                "name": "Lightmatter",
                "url": origin,
                "inLanguage": lang.html(),
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
                    tx(lang, "Ajuste variacional de ε₂ con base triangular", "Variational ε₂ fit on a triangular basis", "用三角基做 ε₂ 的变分拟合", "Ajuste variacional de ε₂ com base triangular"),
                    tx(lang, "ε₁ por Kramers–Kronig discreta", "ε₁ from the discrete Kramers–Kronig transform", "由离散克拉默斯–克罗尼格得到 ε₁", "ε₁ pela Kramers–Kronig discreta"),
                    tx(lang, "n, k, R, α, conductividad, transmitancia y absorbancia", "n, k, R, α, conductivity, transmittance, and absorbance", "n、k、R、α、电导率、透射率和吸光度", "n, k, R, α, condutividade, transmitância e absorbância"),
                    tx(lang, "Luminiscencia: picos, centroide y forma de absorción", "Luminescence: peaks, centroid, and absorption shape", "发光：峰、质心和吸收线形", "Luminescência: picos, centroide e forma da absorção"),
                    tx(lang, "Funciona instalada y sin red", "Works installed and offline", "安装后可离线使用", "Funciona instalado e sem rede")
                ],
                "description": home_description(lang)
            },
            {
                "@type": "HowTo",
                "name": tx(lang, "Ajustar un espectro con Lightmatter", "Fit a spectrum with Lightmatter", "用 Lightmatter 拟合光谱", "Ajustar um espectro com o Lightmatter"),
                "description": tx(lang, "Cargar la medida, decir qué es y leer ε₁, ε₂ y las constantes que salen del par.", "Load the measurement, say what it is, and read ε₁, ε₂, and the constants that follow from the pair.", "载入测量、说明它是什么，并读出 ε₁、ε₂ 以及由这对量得到的常数。", "Carregar a medida, dizer o que é e ler ε₁, ε₂ e as constantes que saem do par."),
                "totalTime": "PT2M",
                "step": [
                    {
                        "@type": "HowToStep",
                        "position": 1,
                        "name": tx(lang, "Cargar el espectro", "Load the spectrum", "载入光谱", "Carregar o espectro"),
                        "text": tx(lang, "En /lab suelta un archivo o pega dos columnas: ω y lo que mediste. Elige la magnitud y la unidad.", "In /lab drop a file or paste two columns: ω and what you measured. Choose the quantity and the unit.", "在 /lab 放下文件或粘贴两列：ω 和测量值。选择物理量和单位。", "Em /lab solte um arquivo ou cole duas colunas: ω e o que você mediu. Escolha a grandeza e a unidade.")
                    },
                    {
                        "@type": "HowToStep",
                        "position": 2,
                        "name": tx(lang, "Ajustar", "Fit", "拟合", "Ajustar"),
                        "text": tx(lang, "Levenberg–Marquardt busca el ε₂ cuya Kramers–Kronig reproduce tu columna. ε∞ vacío se estima o se ajusta según la medida.", "Levenberg–Marquardt looks for the ε₂ whose Kramers–Kronig transform reproduces your column. A blank ε∞ is estimated or fitted according to the measurement.", "Levenberg–Marquardt 寻找其克拉默斯–克罗尼格变换能再现你那一列的 ε₂。ε∞ 留空时按测量估计或拟合。", "Levenberg–Marquardt procura o ε₂ cuja Kramers–Kronig reproduz a sua coluna. ε∞ vazio é estimado ou ajustado conforme a medida.")
                    },
                    {
                        "@type": "HowToStep",
                        "position": 3,
                        "name": tx(lang, "Leer las constantes", "Read the constants", "读取常数", "Ler as constantes"),
                        "text": tx(lang, "De ese par salen n, k, R, la pérdida, α, σ₁ y σ₂. Con espesor salen T y la absorbancia. La guía está en /learn.", "From that pair come n, k, R, the loss, α, σ₁, and σ₂. With a thickness, T and the absorbance. The guide is at /learn.", "由这对量得到 n、k、R、能量损失函数、α、σ₁ 和 σ₂。有厚度时还有 T 和吸光度。指南在 /learn。", "Desse par saem n, k, R, a perda, α, σ₁ e σ₂. Com espessura saem T e a absorbância. O guia está em /learn.")
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

pub fn webpage_json_ld(name: &str, description: &str, path: &str, lang: Lang) -> String {
    let origin = origin();
    let doc = json!({
        "@context": "https://schema.org",
        "@type": "WebPage",
        "name": name,
        "description": description,
        "url": canonical(path),
        "inLanguage": lang.html(),
        "isPartOf": { "@id": format!("{origin}/#website") }
    });
    serde_json::to_string(&doc).unwrap_or_else(|_| "{}".into())
}
