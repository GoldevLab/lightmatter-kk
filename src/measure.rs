//! Qué se midió y en qué unidades. Convierte ε₁, ε₂ en la magnitud medida,
//! busca picos y saca lo que la luminiscencia permite sin Kramers–Kronig.

use std::f64::consts::PI;

/// hc en eV·nm.
pub const HC_EV_NM: f64 = 1239.841_93;
/// Energía de 1 cm⁻¹ en eV.
pub const EV_PER_CM: f64 = 1.239_841_93e-4;
/// Constante de Boltzmann en eV/K.
pub const K_B_EV: f64 = 8.617_333e-5;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measure {
    #[default]
    Eps2,
    Eps1,
    N,
    K,
    R,
    Alpha,
    T,
    Lum,
}

impl Measure {
    pub const ALL: [Measure; 8] = [
        Measure::Eps2,
        Measure::Eps1,
        Measure::N,
        Measure::K,
        Measure::R,
        Measure::Alpha,
        Measure::T,
        Measure::Lum,
    ];

    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim();
        Self::ALL.into_iter().find(|m| m.key() == raw).unwrap_or_default()
    }

    pub fn key(self) -> &'static str {
        match self {
            Measure::Eps2 => "eps2",
            Measure::Eps1 => "eps1",
            Measure::N => "n",
            Measure::K => "k",
            Measure::R => "r",
            Measure::Alpha => "alpha",
            Measure::T => "t",
            Measure::Lum => "lum",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Measure::Eps2 => "ε₂",
            Measure::Eps1 => "ε₁",
            Measure::N => "n",
            Measure::K => "k",
            Measure::R => "R",
            Measure::Alpha => "α",
            Measure::T => "T",
            Measure::Lum => "I",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Measure::Eps2 => "ε₂ — parte imaginaria de la dieléctrica",
            Measure::Eps1 => "ε₁ — parte real de la dieléctrica",
            Measure::N => "n — índice de refracción",
            Measure::K => "k — coeficiente de extinción",
            Measure::R => "R — reflectividad a incidencia normal",
            Measure::Alpha => "α — absorción (cm⁻¹)",
            Measure::T => "T — transmitancia de una lámina",
            Measure::Lum => "Luminiscencia (PL, EL, CL)",
        }
    }

    /// Las que pasan por Kramers–Kronig.
    pub fn uses_kk(self) -> bool {
        self != Measure::Lum
    }

    pub fn needs_wavenumber(self) -> bool {
        matches!(self, Measure::Alpha | Measure::T)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    #[default]
    Other,
    Cm,
    Ev,
    Nm,
}

impl Unit {
    pub const ALL: [Unit; 4] = [Unit::Cm, Unit::Ev, Unit::Nm, Unit::Other];

    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim();
        Self::ALL.into_iter().find(|u| u.key() == raw).unwrap_or_default()
    }

    pub fn key(self) -> &'static str {
        match self {
            Unit::Other => "otra",
            Unit::Cm => "cm",
            Unit::Ev => "ev",
            Unit::Nm => "nm",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Unit::Other => "otra / sin unidad",
            Unit::Cm => "cm⁻¹ (número de onda)",
            Unit::Ev => "eV (energía)",
            Unit::Nm => "nm (longitud de onda)",
        }
    }

    /// Nombre del eje ya convertido. Los nm se pasan a eV antes del ajuste.
    pub fn axis(self) -> &'static str {
        match self {
            Unit::Other => "ω",
            Unit::Cm => "ω (cm⁻¹)",
            Unit::Ev | Unit::Nm => "E (eV)",
        }
    }

    /// Número de onda en cm⁻¹ del eje ya convertido.
    pub fn wavenumber(self, x: f64) -> Option<f64> {
        match self {
            Unit::Other => None,
            Unit::Cm => Some(x),
            Unit::Ev | Unit::Nm => Some(x / EV_PER_CM),
        }
    }

    /// Energía en eV del eje ya convertido.
    pub fn energy(self, x: f64) -> Option<f64> {
        match self {
            Unit::Other => None,
            Unit::Cm => Some(x * EV_PER_CM),
            Unit::Ev | Unit::Nm => Some(x),
        }
    }

    pub fn known(self) -> bool {
        self != Unit::Other
    }
}

/// Pasa λ (nm) a E (eV). La intensidad por intervalo de λ se multiplica por λ²/hc
/// para que sea intensidad por intervalo de energía; el error, igual.
pub fn nm_to_ev(x: &mut [f64], y: &mut [f64], sigma: &mut [f64], per_interval: bool) -> Result<(), String> {
    for i in 0..x.len() {
        let lambda = x[i];
        if !(lambda > 0.0) {
            return Err("la longitud de onda tiene que ser positiva".into());
        }
        x[i] = HC_EV_NM / lambda;
        if per_interval {
            let jac = lambda * lambda / HC_EV_NM;
            y[i] *= jac;
            if let Some(s) = sigma.get_mut(i) {
                *s *= jac;
            }
        }
    }
    Ok(())
}

/// n, k y R a partir de ε. k lleva el signo de ε₂.
pub fn optics(e1: f64, e2: f64) -> (f64, f64, f64) {
    let mag = (e1 * e1 + e2 * e2).sqrt();
    let n = ((mag + e1) / 2.0).max(0.0).sqrt();
    let mut k = ((mag - e1) / 2.0).max(0.0).sqrt();
    if e2 < 0.0 {
        k = -k;
    }
    let nm = n - 1.0;
    let np = n + 1.0;
    let r = (nm * nm + k * k) / (np * np + k * k);
    (n, k, r)
}

/// Magnitud medida en función de ε. `nu` en cm⁻¹, `d_cm` el espesor de la lámina.
pub fn quantity(m: Measure, e1: f64, e2: f64, nu: f64, d_cm: f64) -> f64 {
    match m {
        Measure::Eps2 | Measure::Lum => e2,
        Measure::Eps1 => e1,
        _ => {
            let (n, k, r) = optics(e1, e2);
            match m {
                Measure::N => n,
                Measure::K => k,
                Measure::R => r,
                Measure::Alpha => 4.0 * PI * nu * k,
                Measure::T => {
                    let x = (-4.0 * PI * nu * k * d_cm).exp();
                    let den = 1.0 - r * r * x * x;
                    if den.abs() < 1e-300 {
                        0.0
                    } else {
                        (1.0 - r) * (1.0 - r) * x / den
                    }
                }
                _ => e2,
            }
        }
    }
}

/// Q y sus derivadas respecto de ε₁ y ε₂ (diferencia central).
pub fn quantity_grad(m: Measure, e1: f64, e2: f64, nu: f64, d_cm: f64) -> (f64, f64, f64) {
    match m {
        Measure::Eps2 | Measure::Lum => (e2, 0.0, 1.0),
        Measure::Eps1 => (e1, 1.0, 0.0),
        _ => {
            let q = quantity(m, e1, e2, nu, d_cm);
            let h1 = 1e-6 * (1.0 + e1.abs());
            let h2 = 1e-6 * (1.0 + e2.abs());
            let d1 = (quantity(m, e1 + h1, e2, nu, d_cm) - quantity(m, e1 - h1, e2, nu, d_cm)) / (2.0 * h1);
            let d2 = (quantity(m, e1, e2 + h2, nu, d_cm) - quantity(m, e1, e2 - h2, nu, d_cm)) / (2.0 * h2);
            (q, d1, d2)
        }
    }
}

/// ε₂ de partida en cada dato. `base` es ε∞.
pub fn seed_eps2(m: Measure, w: &[f64], y: &[f64], nu: &[f64], d_cm: f64, base: f64) -> Vec<f64> {
    let n0 = base.max(1.0).sqrt();
    let from_k = |k: f64| 2.0 * n0 * k.max(0.0);
    match m {
        Measure::Eps2 | Measure::Lum => y.to_vec(),
        Measure::Eps1 => vec![0.0; y.len()],
        Measure::N => vec![0.1; y.len()],
        Measure::K => y.iter().map(|&k| from_k(k)).collect(),
        Measure::Alpha => y
            .iter()
            .zip(nu)
            .map(|(&a, &v)| if v > 0.0 { from_k(a / (4.0 * PI * v)) } else { 0.0 })
            .collect(),
        Measure::T => {
            let r0 = ((n0 - 1.0) / (n0 + 1.0)).powi(2);
            let edge = (1.0 - r0) * (1.0 - r0);
            y.iter()
                .zip(nu)
                .map(|(&t, &v)| {
                    let ratio = (t / edge).clamp(1e-12, 1.0);
                    if v > 0.0 && d_cm > 0.0 {
                        from_k(-ratio.ln() / (4.0 * PI * v * d_cm))
                    } else {
                        0.0
                    }
                })
                .collect()
        }
        Measure::R => reflectivity_seed(w, y),
    }
}

/// Fase de la reflectividad por Kramers–Kronig, con R constante fuera del rango.
/// Con r = (1 − N)/(1 + N) la fase física es la de la integral menos π.
fn reflectivity_seed(w: &[f64], y: &[f64]) -> Vec<f64> {
    let n = w.len();
    let l: Vec<f64> = y.iter().map(|r| 0.5 * r.max(1e-9).ln()).collect();
    let (lo, hi) = (w[0], w[n - 1]);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let om = w[i];
        let mut s = 0.0;
        let f = |x: f64, lx: f64| if (x - om).abs() < 1e-12 * om.max(1.0) { 0.0 } else { (lx - l[i]) / (x * x - om * om) };
        for j in 0..n - 1 {
            s += 0.5 * (f(w[j], l[j]) + f(w[j + 1], l[j + 1])) * (w[j + 1] - w[j]);
        }
        if om > lo {
            s += (l[0] - l[i]) / (2.0 * om) * ((om - lo) / (om + lo)).ln();
        }
        if hi > om {
            s += (l[n - 1] - l[i]) / (2.0 * om) * ((hi + om) / (hi - om)).ln();
        }
        let theta = -(2.0 * om / PI) * s;
        let rr = l[i].exp();
        let re = -rr * theta.cos();
        let im = -rr * theta.sin();
        let den = (1.0 + re) * (1.0 + re) + im * im;
        let nr = ((1.0 - re) * (1.0 + re) - im * im) / den;
        let ni = (-im * (1.0 + re) - (1.0 - re) * im) / den;
        out.push((2.0 * nr * ni).max(0.0));
    }
    out
}

/// ε∞ que sugiere el extremo alto de la medida, cuando nadie lo escribió.
pub fn guess_eps_inf(m: Measure, y: &[f64]) -> Option<f64> {
    let last = *y.last()?;
    match m {
        Measure::R => {
            let s = last.clamp(0.0, 0.95).sqrt();
            let n = (1.0 + s) / (1.0 - s);
            Some((n * n).clamp(1.0, 50.0))
        }
        Measure::N => Some((last * last).clamp(1.0, 50.0)),
        Measure::Eps1 => Some(last),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Peak {
    pub position: f64,
    pub height: f64,
    pub prominence: f64,
    /// Ancho a media altura sobre la base local. `None` si la mitad no se alcanza dentro del rango.
    pub fwhm: Option<f64>,
    pub area: f64,
}

fn smooth(y: &[f64], half: usize) -> Vec<f64> {
    if half == 0 {
        return y.to_vec();
    }
    let n = y.len();
    (0..n)
        .map(|i| {
            let a = i.saturating_sub(half);
            let b = (i + half).min(n - 1);
            y[a..=b].iter().sum::<f64>() / (b - a + 1) as f64
        })
        .collect()
}

fn cross(x0: f64, y0: f64, x1: f64, y1: f64, level: f64) -> f64 {
    if (y1 - y0).abs() < 1e-300 {
        x0
    } else {
        x0 + (level - y0) * (x1 - x0) / (y1 - y0)
    }
}

/// Máximos con prominencia de al menos el 5 % del rango, los 12 más prominentes, ordenados por posición.
pub fn find_peaks(x: &[f64], y: &[f64]) -> Vec<Peak> {
    let n = x.len().min(y.len());
    if n < 5 {
        return Vec::new();
    }
    let ys = smooth(&y[..n], if n > 200 { n / 200 } else { 0 });
    let lo = ys.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = hi - lo;
    if !(range > 0.0) {
        return Vec::new();
    }
    let mut peaks = Vec::new();
    let mut i = 1;
    while i < n - 1 {
        if ys[i] > ys[i - 1] {
            let mut j = i;
            while j + 1 < n && ys[j + 1] == ys[i] {
                j += 1;
            }
            if j + 1 < n && ys[j + 1] < ys[i] {
                let top = (i + j) / 2;
                let h = ys[top];
                let mut l = top;
                let mut lmin = h;
                let mut li = top;
                while l > 0 && ys[l - 1] <= h {
                    l -= 1;
                    if ys[l] < lmin {
                        lmin = ys[l];
                        li = l;
                    }
                }
                let mut r = top;
                let mut rmin = h;
                let mut ri = top;
                while r + 1 < n && ys[r + 1] <= h {
                    r += 1;
                    if ys[r] < rmin {
                        rmin = ys[r];
                        ri = r;
                    }
                }
                let base = lmin.max(rmin);
                let prominence = h - base;
                if prominence >= 0.05 * range {
                    let level = base + prominence / 2.0;
                    let mut a = top;
                    while a > li && ys[a] > level {
                        a -= 1;
                    }
                    let left = if ys[a] <= level && a < top { Some(cross(x[a], ys[a], x[a + 1], ys[a + 1], level)) } else { None };
                    let mut b = top;
                    while b < ri && ys[b] > level {
                        b += 1;
                    }
                    let right = if ys[b] <= level && b > top { Some(cross(x[b - 1], ys[b - 1], x[b], ys[b], level)) } else { None };
                    let fwhm = match (left, right) {
                        (Some(a), Some(b)) => Some((b - a).abs()),
                        _ => None,
                    };
                    let mut area = 0.0;
                    for k in li..ri {
                        let ya = (ys[k] - base).max(0.0);
                        let yb = (ys[k + 1] - base).max(0.0);
                        area += 0.5 * (ya + yb) * (x[k + 1] - x[k]);
                    }
                    peaks.push(Peak { position: x[top], height: h, prominence, fwhm, area: area.abs() });
                }
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }
    peaks.sort_by(|a, b| b.prominence.partial_cmp(&a.prominence).unwrap_or(std::cmp::Ordering::Equal));
    peaks.truncate(12);
    peaks.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap_or(std::cmp::Ordering::Equal));
    peaks
}

/// Integral por trapecios y centroide Σ x·y / Σ y.
pub fn integral_and_centroid(x: &[f64], y: &[f64]) -> (f64, Option<f64>) {
    let mut area = 0.0;
    let mut moment = 0.0;
    for i in 0..x.len().min(y.len()).saturating_sub(1) {
        let dx = x[i + 1] - x[i];
        area += 0.5 * (y[i] + y[i + 1]) * dx;
        moment += 0.5 * (x[i] * y[i] + x[i + 1] * y[i + 1]) * dx;
    }
    let centroid = if area.abs() > 1e-300 { Some(moment / area) } else { None };
    (area, centroid)
}

/// Forma relativa de la absorción por van Roosbroeck–Shockley: α ∝ I · e^{E/kT} / E².
/// Solo donde I pasa del 2 % del máximo; normalizada a 1.
pub fn roosbroeck_shockley(e: &[f64], i: &[f64], temperature: f64) -> (Vec<f64>, Vec<f64>) {
    let kt = K_B_EV * temperature;
    let top = i.iter().copied().fold(0.0, f64::max);
    if !(kt > 0.0) || !(top > 0.0) {
        return (Vec::new(), Vec::new());
    }
    let mut xs = Vec::new();
    let mut ln = Vec::new();
    for (&en, &v) in e.iter().zip(i) {
        if en > 0.0 && v > 0.02 * top {
            xs.push(en);
            ln.push(v.ln() - 2.0 * en.ln() + en / kt);
        }
    }
    let peak = ln.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let ys = ln.iter().map(|v| (v - peak).exp()).collect();
    (xs, ys)
}

/// Temperatura efectiva de la cola de alta energía: ln(I/E²) cae con pendiente −1/kT
/// si la absorción es casi constante allí. Usa el tramo entre el 50 % y el 5 % del máximo.
pub fn tail_temperature(e: &[f64], i: &[f64]) -> Option<f64> {
    let n = e.len().min(i.len());
    let (imax, top) = i[..n].iter().copied().enumerate().fold((0, f64::NEG_INFINITY), |acc, (k, v)| if v > acc.1 { (k, v) } else { acc });
    if !(top > 0.0) {
        return None;
    }
    let mut pts = Vec::new();
    for k in imax..n {
        let v = i[k];
        if v < 0.05 * top {
            break;
        }
        if v <= 0.5 * top && e[k] > 0.0 {
            pts.push((e[k], v.ln() - 2.0 * e[k].ln()));
        }
    }
    if pts.len() < 4 {
        return None;
    }
    let m = pts.len() as f64;
    let sx: f64 = pts.iter().map(|p| p.0).sum();
    let sy: f64 = pts.iter().map(|p| p.1).sum();
    let sxx: f64 = pts.iter().map(|p| p.0 * p.0).sum();
    let sxy: f64 = pts.iter().map(|p| p.0 * p.1).sum();
    let den = m * sxx - sx * sx;
    if den.abs() < 1e-300 {
        return None;
    }
    let slope = (m * sxy - sx * sy) / den;
    // Solo si la cola es de verdad una recta: otra banda encima o una gaussiana la curvan.
    let mean = sy / m;
    let intercept = (sy - slope * sx) / m;
    let ss_tot: f64 = pts.iter().map(|p| (p.1 - mean).powi(2)).sum();
    let ss_res: f64 = pts.iter().map(|p| (p.1 - intercept - slope * p.0).powi(2)).sum();
    let r2 = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };
    if slope < 0.0 && r2 >= 0.97 {
        let t = -1.0 / (K_B_EV * slope);
        if t.is_finite() && t < 5000.0 {
            return Some(t);
        }
    }
    None
}

/// Corte de Tauc y qué tan recta es la subida. p = 2 directa, ½ indirecta.
#[derive(Clone, Copy, Debug)]
pub struct TaucEdge {
    pub gap: f64,
    pub r2: f64,
}

/// Brecha por la tangente más empinada de (α·E)^p antes de su máximo. p = 2 directa, ½ indirecta.
pub fn tauc_gap(e: &[f64], alpha: &[f64], p: f64) -> Option<f64> {
    let n = e.len().min(alpha.len());
    if n < 5 {
        return None;
    }
    let t: Vec<f64> = (0..n).map(|k| (alpha[k].max(0.0) * e[k]).powf(p)).collect();
    let (kmax, tmax) = t.iter().copied().enumerate().fold((0, f64::NEG_INFINITY), |acc, (k, v)| if v > acc.1 { (k, v) } else { acc });
    // Un borde electrónico: energías de al menos 0.2 eV y absorción que arranca casi en cero.
    if !(tmax > 0.0) || kmax < 2 || e[0] < 0.2 || t[0] > 0.05 * tmax {
        return None;
    }
    let mut best: Option<(f64, usize)> = None;
    for k in 1..kmax {
        if t[k] < 0.1 * tmax || t[k] > 0.9 * tmax {
            continue;
        }
        let s = (t[k + 1] - t[k - 1]) / (e[k + 1] - e[k - 1]);
        if s > 0.0 && best.is_none_or(|(b, _)| s > b) {
            best = Some((s, k));
        }
    }
    let (s, k) = best?;
    let gap = e[k] - t[k] / s;
    if gap > e[0] - (e[n - 1] - e[0]) && gap < e[k] {
        Some(gap)
    } else {
        None
    }
}

/// Compara la linealidad de ambos exponentes en la misma ventana del borde.
pub fn tauc_edge(e: &[f64], alpha: &[f64], p: f64) -> Option<TaucEdge> {
    let gap = tauc_gap(e, alpha, p)?;
    Some(TaucEdge { gap, r2: flank_r2(e, alpha, p).unwrap_or(0.0) })
}

fn flank_r2(e: &[f64], alpha: &[f64], p: f64) -> Option<f64> {
    let n = e.len().min(alpha.len());
    if n < 5 {
        return None;
    }
    let base: Vec<f64> = (0..n).map(|k| alpha[k].max(0.0) * e[k]).collect();
    let (kmax, base_max) = base.iter().copied().enumerate().fold((0, f64::NEG_INFINITY), |acc, (k, v)| if v > acc.1 { (k, v) } else { acc });
    if !(base_max > 0.0) || kmax < 4 {
        return None;
    }
    let pts: Vec<(f64, f64)> = (0..kmax)
        .filter(|&k| base[k] >= 0.05 * base_max && base[k] <= 0.70 * base_max)
        .map(|k| (e[k], base[k].powf(p)))
        .collect();
    if pts.len() < 5 {
        return None;
    }
    Some(lin_r2(&pts))
}

fn lin_r2(pts: &[(f64, f64)]) -> f64 {
    let m = pts.len() as f64;
    let sx: f64 = pts.iter().map(|p| p.0).sum();
    let sy: f64 = pts.iter().map(|p| p.1).sum();
    let sxx: f64 = pts.iter().map(|p| p.0 * p.0).sum();
    let sxy: f64 = pts.iter().map(|p| p.0 * p.1).sum();
    let den = m * sxx - sx * sx;
    if den.abs() < 1e-300 {
        return 0.0;
    }
    let slope = (m * sxy - sx * sy) / den;
    let intercept = (sy - slope * sx) / m;
    let mean = sy / m;
    let ss_tot: f64 = pts.iter().map(|p| (p.1 - mean).powi(2)).sum();
    if ss_tot < 1e-300 {
        return 0.0;
    }
    let ss_res: f64 = pts.iter().map(|p| {
        let y = intercept + slope * p.0;
        (p.1 - y).powi(2)
    }).sum();
    (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
}

/// Qué fórmula mostrar y la frase que la explica.
/// `rising` es cierto si α sigue en su máximo al final del rango.
pub fn tauc_account(direct: Option<TaucEdge>, indirect: Option<TaucEdge>, rising: bool) -> (&'static str, String) {
    let tail = if rising {
        " α sigue en su máximo al final del rango: el corte es la tangente de esa subida y queda por encima del inicio visible del borde."
    } else {
        ""
    };
    let r2s = |v: f64| format!("{v:.2}");
    match (direct, indirect) {
        (None, None) => ("", String::new()),
        (Some(d), None) => {
            let show = if d.r2 >= 0.90 { "direct" } else { "rough" };
            let note = if d.r2 >= 0.90 {
                format!("Este borde sigue la fórmula directa: (αE)² es una recta (R² = {}).{tail}", r2s(d.r2))
            } else {
                format!("El corte directo es orientativo: (αE)² no es una recta limpia (R² = {}).{tail}", r2s(d.r2))
            };
            (show, note)
        }
        (None, Some(i)) => {
            let show = if i.r2 >= 0.90 { "indirect" } else { "rough" };
            let note = if i.r2 >= 0.90 {
                format!("Este borde sigue la fórmula indirecta: (αE)^½ es una recta (R² = {}).{tail}", r2s(i.r2))
            } else {
                format!("El corte indirecto es orientativo: (αE)^½ no es una recta limpia (R² = {}).{tail}", r2s(i.r2))
            };
            (show, note)
        }
        (Some(d), Some(i)) => {
            let gap = |v: f64| format!("{v:.4}");
            if d.r2 >= 0.90 && d.r2 >= i.r2 + 0.08 {
                (
                    "direct",
                    format!(
                        "La brecha que describe este borde es la directa: (αE)² es una recta (R² = {}). La fórmula indirecta corta en {} eV y (αE)^½ no es recta aquí (R² = {}).{tail}",
                        r2s(d.r2),
                        gap(i.gap),
                        r2s(i.r2)
                    ),
                )
            } else if i.r2 >= 0.90 && i.r2 >= d.r2 + 0.08 {
                (
                    "indirect",
                    format!(
                        "La brecha que describe este borde es la indirecta: (αE)^½ es una recta (R² = {}). La fórmula directa corta en {} eV y (αE)² no es recta aquí (R² = {}).{tail}",
                        r2s(i.r2),
                        gap(d.gap),
                        r2s(d.r2)
                    ),
                )
            } else if d.r2 >= 0.90 && i.r2 >= 0.90 {
                (
                    "both",
                    format!(
                        "Las dos fórmulas de Tauc son rectas en este borde (R² directa {}, indirecta {}). El corte no dice si la brecha es directa o indirecta: eso lo dice el material.{tail}",
                        r2s(d.r2),
                        r2s(i.r2)
                    ),
                )
            } else {
                (
                    "rough",
                    format!(
                        "Ninguna fórmula de Tauc es una recta limpia (R² directa {}, indirecta {}). El número es orientativo.{tail}",
                        r2s(d.r2),
                        r2s(i.r2)
                    ),
                )
            }
        }
    }
}

/// Máximo pegado al extremo alto, sin bajada después. Devuelve (índice, altura).
pub fn high_end_maximum(y: &[f64]) -> Option<(usize, f64)> {
    let n = y.len();
    if n < 8 {
        return None;
    }
    let (imax, ymax) = y.iter().enumerate().fold((0, f64::NEG_INFINITY), |acc, (i, &v)| if v > acc.1 { (i, v) } else { acc });
    if !(ymax > 0.0) {
        return None;
    }
    let tail = (n / 12).max(2);
    if imax + 1 >= n - tail && y[n - 1] >= 0.85 * ymax {
        Some((imax, ymax))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaussian_peak_width_and_area() {
        let x: Vec<f64> = (0..400).map(|i| 1.0 + i as f64 * 0.005).collect();
        let s = 0.05_f64;
        let y: Vec<f64> = x.iter().map(|v| (-(v - 2.0) * (v - 2.0) / (2.0 * s * s)).exp()).collect();
        let peaks = find_peaks(&x, &y);
        assert_eq!(peaks.len(), 1);
        let p = &peaks[0];
        assert!((p.position - 2.0).abs() < 0.006);
        let fwhm = 2.0 * (2.0 * 2f64.ln()).sqrt() * s;
        assert!((p.fwhm.unwrap() - fwhm).abs() < 0.005, "fwhm {:?}", p.fwhm);
        let area = s * (2.0 * PI).sqrt();
        assert!((p.area - area).abs() < 0.003, "área {}", p.area);
    }

    #[test]
    fn nm_becomes_ev_with_the_jacobian() {
        let mut x = vec![500.0];
        let mut y = vec![1.0];
        let mut s = vec![0.1];
        nm_to_ev(&mut x, &mut y, &mut s, true).unwrap();
        assert!((x[0] - 2.479_683_86).abs() < 1e-6);
        assert!((y[0] - 500.0 * 500.0 / HC_EV_NM).abs() < 1e-9);
    }

    #[test]
    fn tail_gives_back_the_temperature() {
        let t = 300.0;
        let e: Vec<f64> = (0..300).map(|i| 1.3 + i as f64 * 0.002).collect();
        let i: Vec<f64> = e
            .iter()
            .map(|&en| if en < 1.42 { 0.0 } else { en * en * (-(en - 1.42) / (K_B_EV * t)).exp() })
            .collect();
        let got = tail_temperature(&e, &i).unwrap();
        assert!((got - t).abs() < 3.0, "T {got}");
    }

    #[test]
    fn direct_gap_from_a_square_root_edge() {
        let e: Vec<f64> = (0..200).map(|i| 1.0 + i as f64 * 0.01).collect();
        let a: Vec<f64> = e.iter().map(|&en| if en > 1.5 { (en - 1.5).sqrt() / en } else { 0.0 }).collect();
        let a2: Vec<f64> = a.iter().zip(&e).map(|(v, en)| v * if *en < 2.6 { 1.0 } else { (3.0 - en).max(0.0) }).collect();
        let gap = tauc_gap(&e, &a2, 2.0).unwrap();
        assert!((gap - 1.5).abs() < 0.03, "gap {gap}");
        let direct = tauc_edge(&e, &a2, 2.0).unwrap();
        let indirect = tauc_edge(&e, &a2, 0.5).unwrap();
        assert!(direct.r2 > indirect.r2 + 0.08, "R² directa {} indirecta {}", direct.r2, indirect.r2);
        let (show, note) = tauc_account(Some(direct), Some(indirect), false);
        assert_eq!(show, "direct");
        assert!(note.contains("directa"), "{note}");
    }

    #[test]
    fn indirect_edge_prefers_the_square_root_plot() {
        let e: Vec<f64> = (0..200).map(|i| 1.0 + i as f64 * 0.01).collect();
        let a: Vec<f64> = e
            .iter()
            .map(|&en| {
                if en <= 1.5 {
                    0.0
                } else if en < 2.6 {
                    (en - 1.5).powi(2) / en
                } else {
                    (2.6_f64 - 1.5).powi(2) / en * ((3.0 - en) / 0.4).max(0.0)
                }
            })
            .collect();
        let direct = tauc_edge(&e, &a, 2.0).unwrap();
        let indirect = tauc_edge(&e, &a, 0.5).unwrap();
        assert!(indirect.r2 > direct.r2 + 0.08, "R² directa {} indirecta {}", direct.r2, indirect.r2);
        assert_eq!(tauc_account(Some(direct), Some(indirect), true).0, "indirect");
    }

    #[test]
    fn transmittance_of_a_clear_slab() {
        let t = quantity(Measure::T, 2.25, 0.0, 1000.0, 1e-4);
        let r = (0.5_f64 / 2.5).powi(2);
        assert!((t - (1.0 - r) / (1.0 + r)).abs() < 1e-9);
    }
}
