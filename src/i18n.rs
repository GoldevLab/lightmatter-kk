//! Idioma de la interfaz. La elección guardada manda; si no hay, se negocia
//! `Accept-Language` (RFC 9110). Lo que no está entre estos cuatro cae en español.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Es,
    En,
    Zh,
    Pt,
}

impl Lang {
    pub const ALL: [Lang; 4] = [Lang::Es, Lang::En, Lang::Zh, Lang::Pt];

    pub fn code(self) -> &'static str {
        match self {
            Lang::Es => "es",
            Lang::En => "en",
            Lang::Zh => "zh",
            Lang::Pt => "pt",
        }
    }

    /// Valor de `<html lang>`. El chino de la interfaz es simplificado.
    pub fn html(self) -> &'static str {
        match self {
            Lang::Es => "es",
            Lang::En => "en",
            Lang::Zh => "zh-Hans",
            Lang::Pt => "pt",
        }
    }

    pub fn locale(self) -> &'static str {
        match self {
            Lang::Es => "es_ES",
            Lang::En => "en_US",
            Lang::Zh => "zh_CN",
            Lang::Pt => "pt_BR",
        }
    }

    /// Nombre del idioma en su propia lengua, para el selector.
    pub fn menu_label(self) -> &'static str {
        match self {
            Lang::Es => "Español",
            Lang::En => "English",
            Lang::Zh => "中文",
            Lang::Pt => "Português",
        }
    }

    pub fn from_code(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "es" => Some(Lang::Es),
            "en" => Some(Lang::En),
            "zh" | "zh-hans" | "zh-cn" => Some(Lang::Zh),
            "pt" | "pt-br" => Some(Lang::Pt),
            _ => None,
        }
    }
}

pub fn tx(lang: Lang, es: &'static str, en: &'static str, zh: &'static str, pt: &'static str) -> &'static str {
    match lang {
        Lang::Es => es,
        Lang::En => en,
        Lang::Zh => zh,
        Lang::Pt => pt,
    }
}

/// Cookie `lm_lang` si es uno de los cuatro. Si no, `Accept-Language`.
pub fn resolve(cookie_header: Option<&str>, accept_language: Option<&str>) -> Lang {
    if let Some(lang) = cookie_pick(cookie_header) {
        return lang;
    }
    negotiate(accept_language.unwrap_or(""))
}

fn cookie_pick(header: Option<&str>) -> Option<Lang> {
    let header = header?;
    for part in header.split(';') {
        let part = part.trim();
        let Some(value) = part.strip_prefix("lm_lang=") else {
            continue;
        };
        if let Some(lang) = Lang::from_code(value.trim()) {
            return Some(lang);
        }
    }
    None
}

/// Elige el idioma de mayor calidad (`q`) que esta app tenga.
/// Un idioma que no está en la lista no se inventa: se sigue con el siguiente,
/// y si ninguno coincide, español.
pub fn negotiate(header: &str) -> Lang {
    let mut ranges: Vec<(String, f32, usize)> = Vec::new();
    for (index, part) in header.split(',').enumerate() {
        let mut bits = part.split(';');
        let tag = bits.next().unwrap_or("").trim().to_ascii_lowercase();
        if tag.is_empty() {
            continue;
        }
        let mut q = 1.0_f32;
        for param in bits {
            let param = param.trim();
            if let Some(value) = param.strip_prefix("q=") {
                q = value.trim().parse().unwrap_or(0.0);
            }
        }
        if q > 0.0 {
            ranges.push((tag, q, index));
        }
    }
    ranges.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.2.cmp(&b.2)));
    for (tag, _, _) in ranges {
        if tag == "*" {
            continue;
        }
        if let Some(lang) = match_tag(&tag) {
            return lang;
        }
    }
    Lang::Es
}

fn match_tag(tag: &str) -> Option<Lang> {
    let primary = tag.split(['-', '_']).next().unwrap_or(tag);
    match primary {
        "es" => Some(Lang::Es),
        "en" => Some(Lang::En),
        "zh" => Some(Lang::Zh),
        "pt" => Some(Lang::Pt),
        _ => None,
    }
}

pub fn measure_name(lang: Lang, key: &str) -> &'static str {
    match key {
        "eps2" => tx(lang, "ε₂ — parte imaginaria", "ε₂ — imaginary part", "ε₂ — 虚部", "ε₂ — parte imaginária"),
        "eps1" => tx(lang, "ε₁ — parte real", "ε₁ — real part", "ε₁ — 实部", "ε₁ — parte real"),
        "n" => tx(lang, "n — índice de refracción", "n — refractive index", "n — 折射率", "n — índice de refração"),
        "k" => tx(lang, "k — extinción", "k — extinction", "k — 消光系数", "k — extinção"),
        "r" => tx(lang, "R — reflectividad", "R — reflectivity", "R — 反射率", "R — refletividade"),
        "alpha" => tx(lang, "α — absorción (cm⁻¹)", "α — absorption (cm⁻¹)", "α — 吸收系数 (cm⁻¹)", "α — absorção (cm⁻¹)"),
        "t" => tx(lang, "T — transmitancia", "T — transmittance", "T — 透射率", "T — transmitância"),
        "lum" => tx(lang, "Luminiscencia", "Luminescence", "发光", "Luminescência"),
        _ => "—",
    }
}

pub fn unit_name(lang: Lang, key: &str) -> &'static str {
    match key {
        "cm" => tx(lang, "cm⁻¹ (número de onda)", "cm⁻¹ (wavenumber)", "cm⁻¹（波数）", "cm⁻¹ (número de onda)"),
        "ev" => tx(lang, "eV (energía)", "eV (energy)", "eV（能量）", "eV (energia)"),
        "nm" => tx(lang, "nm (longitud de onda)", "nm (wavelength)", "nm（波长）", "nm (comprimento de onda)"),
        _ => tx(lang, "otra / sin unidad", "other / no unit", "其他 / 无单位", "outra / sem unidade"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_order_and_quality() {
        assert_eq!(negotiate("pt-BR,pt;q=0.9,en;q=0.8"), Lang::Pt);
        assert_eq!(negotiate("zh-CN,zh;q=0.9,en;q=0.7"), Lang::Zh);
        assert_eq!(negotiate("en;q=0.2,es;q=0.9"), Lang::Es);
        assert_eq!(negotiate("fr-FR,fr;q=0.9,en;q=0.5"), Lang::En);
        assert_eq!(negotiate("fr-FR,de;q=0.8"), Lang::Es);
        assert_eq!(negotiate(""), Lang::Es);
    }

    #[test]
    fn cookie_wins_over_the_browser() {
        assert_eq!(
            resolve(Some("lm_lang=zh; lm_admin=no"), Some("en-US,en;q=0.9")),
            Lang::Zh
        );
        assert_eq!(resolve(Some("theme=dark"), Some("pt-PT,en;q=0.4")), Lang::Pt);
    }
}
