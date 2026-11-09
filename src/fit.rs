//! Ajuste variacional con restricción de Kramers–Kronig (Kuzmenko, 2005).
//! Levenberg–Marquardt (Numerical Recipes §15.5) sobre la base triangular de ε₂
//! y la transformada KK discreta para ε₁. Fondo Drude opcional.
//! La medida puede ser ε₂, ε₁, n, k, R, α o T: el ajuste compara esa magnitud.

use crate::measure::{
    find_peaks, guess_eps_inf, high_end_maximum, integral_and_centroid, nm_to_ev, optics, quantity, quantity_grad,
    roosbroeck_shockley, seed_eps2, tail_temperature, tauc_account, tauc_edge, Measure, Peak, Unit, EV_PER_CM,
};

/// Qué se midió, en qué unidades, y lo que la física de esa medida necesita.
#[derive(Clone, Debug)]
pub struct Physics {
    pub measure: Measure,
    pub unit: Unit,
    /// Espesor de la lámina para T.
    pub thickness_um: f64,
    /// Temperatura de la muestra para la luminiscencia.
    pub temperature_k: f64,
    /// Vacío: depende de la medida (se estima, se ajusta o vale 1).
    pub eps_inf: Option<f64>,
}

impl Default for Physics {
    fn default() -> Self {
        Self { measure: Measure::Eps2, unit: Unit::Other, thickness_um: 0.0, temperature_k: 300.0, eps_inf: None }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Drude {
    pub eps_inf: f64,
    pub omega_p: f64,
    pub gamma: f64,
}

#[derive(Clone, Debug)]
pub struct FitRequest {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub iter_max: usize,
    pub log_mesh: bool,
    pub drude: Option<Drude>,
    pub smoothness: f64,
    pub negativity: f64,
    pub extended: bool,
    /// Error experimental de cada punto. Vacío: todos valen 1.
    pub sigma: Vec<f64>,
    /// La tercera columna vino en el espectro.
    pub errors_given: bool,
    pub physics: Physics,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FitOptions {
    #[serde(default)]
    pub log_mesh: bool,
    #[serde(default)]
    pub extended: bool,
    #[serde(default = "default_smoothness")]
    pub smoothness: f64,
    #[serde(default = "default_negativity")]
    pub negativity: f64,
    #[serde(default = "default_iter")]
    pub iter_max: usize,
    #[serde(default)]
    pub measure: Measure,
    #[serde(default)]
    pub unit: Unit,
    #[serde(default)]
    pub thickness_um: f64,
    #[serde(default = "default_temperature")]
    pub temperature_k: f64,
    /// ε∞ tal como lo escribió el usuario.
    #[serde(default)]
    pub eps_inf: Option<f64>,
}

fn default_temperature() -> f64 {
    300.0
}

fn default_smoothness() -> f64 {
    0.08
}
fn default_negativity() -> f64 {
    100.0
}
fn default_iter() -> usize {
    20
}

impl Default for FitOptions {
    fn default() -> Self {
        Self {
            log_mesh: false,
            extended: false,
            smoothness: default_smoothness(),
            negativity: default_negativity(),
            iter_max: default_iter(),
            measure: Measure::Eps2,
            unit: Unit::Other,
            thickness_um: 0.0,
            temperature_k: default_temperature(),
            eps_inf: None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FitReport {
    pub omega: Vec<f64>,
    pub eps1: Vec<f64>,
    pub eps2: Vec<f64>,
    pub data_x: Vec<f64>,
    pub data_y: Vec<f64>,
    pub n_index: Vec<f64>,
    pub k_index: Vec<f64>,
    pub reflectivity: Vec<f64>,
    pub ext_omega: Vec<f64>,
    pub ext_eps1: Vec<f64>,
    pub ext_eps2: Vec<f64>,
    pub chisq: f64,
    pub chisq_dof: f64,
    pub n_anchor: usize,
    pub mfit: usize,
    pub points_used: usize,
    pub warnings: Vec<String>,
    pub drude: Option<Drude>,
    /// Pasos de Levenberg–Marquardt que llegaron a ejecutarse.
    #[serde(default)]
    pub iters_used: usize,
    /// χ² dejó de mejorar más de un 1 %, el criterio de parada de este laboratorio.
    #[serde(default)]
    pub converged: bool,
    /// ε₂ del modelo en cada ω de los datos, para comparar punto a punto.
    #[serde(default)]
    pub model_eps2: Vec<f64>,
    /// (modelo − dato) × 100, el mismo residuo que el panel de diferencia del laboratorio Qwik.
    #[serde(default)]
    pub difference: Vec<f64>,
    /// Controles con los que se hizo el ajuste, para repetirlo desde la página.
    #[serde(default)]
    pub options: FitOptions,
    /// Error experimental, alineado con `data_x`. Vacío si no se pegó la tercera columna.
    #[serde(default)]
    pub data_sigma: Vec<f64>,
    /// ±1σ del modelo de ε₂ en cada dato, desde la covarianza. Vacío sin error experimental.
    #[serde(default)]
    pub eps2_err: Vec<f64>,
    /// ±1σ de ε₂ en cada ancla interior. Vacío sin error experimental.
    #[serde(default)]
    pub coef_err: Vec<f64>,
    #[serde(default)]
    pub errors_given: bool,
    /// La magnitud medida según el modelo, en cada dato.
    #[serde(default)]
    pub model_q: Vec<f64>,
    /// ±1σ de esa magnitud. Vacío sin error experimental.
    #[serde(default)]
    pub q_err: Vec<f64>,
    /// −Im(1/ε) en cada ancla.
    #[serde(default)]
    pub loss: Vec<f64>,
    /// σ₁ en Ω⁻¹cm⁻¹. Vacío si ω no tiene unidad.
    #[serde(default)]
    pub sigma1: Vec<f64>,
    /// σ₂ = ν̃(ε∞ − ε₁)/60 en Ω⁻¹cm⁻¹. Vacío si ω no tiene unidad.
    #[serde(default)]
    pub sigma2: Vec<f64>,
    /// α = 4πν̃k en cm⁻¹. Vacío si ω no tiene unidad.
    #[serde(default)]
    pub absorption: Vec<f64>,
    /// Transmitancia de una lámina en aire. Vacío sin espesor o sin unidad de ω.
    #[serde(default)]
    pub transmittance: Vec<f64>,
    /// Absorbancia −log₁₀(T). Vacío cuando no hay transmitancia.
    #[serde(default)]
    pub absorbance: Vec<f64>,
    /// Picos de ε₂ o, en luminiscencia, de la intensidad.
    #[serde(default)]
    pub peaks: Vec<Peak>,
    #[serde(default)]
    pub eps_inf: Option<f64>,
    #[serde(default)]
    pub eps_inf_err: Option<f64>,
    #[serde(default)]
    pub eps_inf_note: String,
    #[serde(default)]
    pub tauc_direct: Option<f64>,
    #[serde(default)]
    pub tauc_indirect: Option<f64>,
    /// R² de la subida en (αE)² y en (αE)^½.
    #[serde(default)]
    pub tauc_direct_r2: Option<f64>,
    #[serde(default)]
    pub tauc_indirect_r2: Option<f64>,
    /// Qué corte mostrar: direct, indirect, both, rough, o vacío.
    #[serde(default)]
    pub tauc_show: String,
    /// Frase que dice cuál fórmula describe el borde.
    #[serde(default)]
    pub tauc_note: String,
    #[serde(default)]
    pub integral: Option<f64>,
    #[serde(default)]
    pub centroid: Option<f64>,
    /// Temperatura efectiva de la cola alta de la luminiscencia, en K.
    #[serde(default)]
    pub carrier_temp: Option<f64>,
    /// Absorción relativa por van Roosbroeck–Shockley.
    #[serde(default)]
    pub vrs_x: Vec<f64>,
    #[serde(default)]
    pub vrs_y: Vec<f64>,
}

impl FitReport {
    fn empty() -> Self {
        Self {
            omega: Vec::new(),
            eps1: Vec::new(),
            eps2: Vec::new(),
            data_x: Vec::new(),
            data_y: Vec::new(),
            n_index: Vec::new(),
            k_index: Vec::new(),
            reflectivity: Vec::new(),
            ext_omega: Vec::new(),
            ext_eps1: Vec::new(),
            ext_eps2: Vec::new(),
            chisq: 0.0,
            chisq_dof: 0.0,
            n_anchor: 0,
            mfit: 0,
            points_used: 0,
            warnings: Vec::new(),
            drude: None,
            iters_used: 0,
            converged: false,
            model_eps2: Vec::new(),
            difference: Vec::new(),
            options: FitOptions::default(),
            data_sigma: Vec::new(),
            eps2_err: Vec::new(),
            coef_err: Vec::new(),
            errors_given: false,
            model_q: Vec::new(),
            q_err: Vec::new(),
            loss: Vec::new(),
            sigma1: Vec::new(),
            sigma2: Vec::new(),
            absorption: Vec::new(),
            transmittance: Vec::new(),
            absorbance: Vec::new(),
            peaks: Vec::new(),
            eps_inf: None,
            eps_inf_err: None,
            eps_inf_note: String::new(),
            tauc_direct: None,
            tauc_indirect: None,
            tauc_direct_r2: None,
            tauc_indirect_r2: None,
            tauc_show: String::new(),
            tauc_note: String::new(),
            integral: None,
            centroid: None,
            carrier_temp: None,
            vrs_x: Vec::new(),
            vrs_y: Vec::new(),
        }
    }

    pub fn measure(&self) -> Measure {
        self.options.measure
    }
}

pub fn parse_spectrum(text: &str) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, bool), String> {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut ss = Vec::new();
    let mut with_error = 0usize;
    let mut without_error = 0usize;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || "#*!%".contains(t.chars().next().unwrap_or(' ')) {
            continue;
        }
        let parts: Vec<&str> = t.split(|c: char| c.is_whitespace() || c == ',' || c == ';').filter(|s| !s.is_empty()).collect();
        if parts.len() < 2 {
            continue;
        }
        let (Ok(a), Ok(b)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) else {
            continue;
        };
        if !(a.is_finite() && b.is_finite()) {
            continue;
        }
        let sigma = if parts.len() >= 3 {
            match parts[2].parse::<f64>() {
                Ok(s) if s.is_finite() && s > 0.0 => Some(s),
                _ => return Err("el error de cada punto tiene que ser un número positivo".into()),
            }
        } else {
            None
        };
        match sigma {
            Some(_) => with_error += 1,
            None => without_error += 1,
        }
        xs.push(a);
        ys.push(b);
        ss.push(sigma.unwrap_or(1.0));
    }
    if xs.len() < 3 {
        return Err("se necesitan al menos 3 filas numéricas (ω y valor)".into());
    }
    if with_error > 0 && without_error > 0 {
        return Err("si escribes el error, ponlo en todas las filas. Es la tercera columna.".into());
    }
    Ok((xs, ys, ss, with_error > 0))
}

pub fn downsample(values: &[f64], target: usize) -> Vec<f64> {
    if target < 2 || values.len() <= target {
        return values.to_vec();
    }
    let last = values.len() - 1;
    let step = last as f64 / (target - 1) as f64;
    (0..target)
        .map(|i| values[((i as f64 * step).round() as usize).min(last)])
        .collect()
}

fn kuzmenko_g(x: f64, y: f64) -> f64 {
    let sp = x + y;
    let sm = x - y;
    let a = if sp == 0.0 { 0.0 } else { sp * sp.abs().ln() };
    let b = if sm == 0.0 { 0.0 } else { sm * sm.abs().ln() };
    a - b
}

fn eps2_drude(omega: f64, m: &Drude) -> f64 {
    if !omega.is_finite() || omega <= 0.0 {
        return 0.0;
    }
    let w2 = omega * omega;
    let den = w2 * w2 + m.gamma * m.gamma * w2;
    if den <= 1e-300 {
        return 0.0;
    }
    (m.omega_p * m.omega_p * m.gamma * omega) / den
}

fn eps1_drude(omega: f64, m: &Drude) -> f64 {
    if !omega.is_finite() || omega <= 0.0 {
        return m.eps_inf;
    }
    let w2 = omega * omega;
    let den = w2 * w2 + m.gamma * m.gamma * w2;
    if den <= 1e-300 {
        return m.eps_inf;
    }
    m.eps_inf - (m.omega_p * m.omega_p * w2) / den
}

pub fn suggest_drude(xs: &[f64], e2: &[f64]) -> Drude {
    let n = xs.len().min(e2.len());
    if n < 4 {
        let gamma = if n >= 2 {
            let lo = xs.iter().copied().fold(f64::INFINITY, f64::min).max(1e-9);
            let hi = xs.iter().copied().fold(0.0, f64::max);
            ((hi - lo).max(lo) * 0.05).max(lo * 0.05)
        } else {
            1.0
        };
        return Drude { eps_inf: 1.0, omega_p: 0.0, gamma };
    }
    let mut ord: Vec<usize> = (0..n).collect();
    ord.sort_by(|&a, &b| xs[a].partial_cmp(&xs[b]).unwrap_or(std::cmp::Ordering::Equal));
    let w: Vec<f64> = ord.iter().map(|&i| xs[i]).collect();
    let y: Vec<f64> = ord.iter().map(|&i| e2[i]).collect();
    let w_min = w[0].max(1e-9);
    let w_max = w[n - 1];
    let span = (w_max - w_min).max(1e-30);
    let lo_count = 3.max(n / 5);
    let mut swe2 = 0.0;
    for i in 0..lo_count {
        swe2 += w[i] * y[i];
    }
    let mean_we2 = swe2 / lo_count as f64;
    // Sin un suelo fijo: un 5 absoluto solo tiene sentido si ω está en cm⁻¹.
    let mut gamma = (span * 0.03).max(w_min * 0.5);
    gamma = gamma.min(w_max * 0.25).max(w_max * 1e-4);
    let omega_p = (mean_we2 * gamma).max(0.0).sqrt().min(w_max * 6.0);
    Drude { eps_inf: 1.0, omega_p, gamma }
}

fn logspace(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    if n < 2 {
        return vec![lo];
    }
    let ln_lo = lo.max(1e-30).ln();
    let ln_hi = hi.max(1e-30).ln();
    let step = (ln_hi - ln_lo) / (n - 1) as f64;
    (0..n).map(|i| (ln_lo + i as f64 * step).exp()).collect()
}

fn tri_basis(omega: f64, j: usize, anch: &[f64]) -> f64 {
    if j == 0 || j + 1 >= anch.len() {
        return 0.0;
    }
    let (wp, wc, wn) = (anch[j - 1], anch[j], anch[j + 1]);
    if omega <= wp || omega >= wn {
        return 0.0;
    }
    let rise = wc - wp;
    let fall = wn - wc;
    if rise.abs() < 1e-30 || fall.abs() < 1e-30 {
        return 0.0;
    }
    if omega <= wc {
        (omega - wp) / rise
    } else {
        (wn - omega) / fall
    }
}

fn interp(omega: f64, xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len();
    if n == 0 {
        return 0.0;
    }
    if omega <= xs[0] {
        return ys[0];
    }
    if omega >= xs[n - 1] {
        return ys[n - 1];
    }
    let mut lo = 0usize;
    let mut hi = n - 1;
    while hi - lo > 1 {
        let m = (lo + hi) >> 1;
        if xs[m] <= omega {
            lo = m;
        } else {
            hi = m;
        }
    }
    let den = xs[hi] - xs[lo];
    let t = if den == 0.0 { 0.0 } else { (omega - xs[lo]) / den };
    ys[lo] + t * (ys[hi] - ys[lo])
}

#[cfg(test)]
fn dkkt_eps1(om: f64, anchors: &[f64], a: &[f64]) -> f64 {
    let ma = anchors.len();
    let mut sum = 0.0;
    for j in 1..ma.saturating_sub(1) {
        sum += a[j] * kk_kernel(om, anchors[j - 1], anchors[j], anchors[j + 1]);
    }
    1.0 + sum
}

/// ε₁ en ω de un triángulo de altura 1 sobre (wm, wj, wp).
fn kk_kernel(om: f64, wm: f64, wj: f64, wp: f64) -> f64 {
    let denom1 = if (wj - wm).abs() < 1e-30 { 1e-30 } else { wj - wm };
    let denom2 = if (wp - wj).abs() < 1e-30 { 1e-30 } else { wp - wj };
    (1.0 / std::f64::consts::PI)
        * (kuzmenko_g(om, wm) / denom1 - ((wp - wm) * kuzmenko_g(om, wj)) / (denom1 * denom2) + kuzmenko_g(om, wp) / denom2)
}

fn mat(n: usize) -> Vec<Vec<f64>> {
    vec![vec![0.0; n]; n]
}

/// Matriz simétrica del paso normal (αᵀ de Numerical Recipes). Solo se escribe el triángulo inferior.
trait SysMat {
    fn size(&self) -> usize;
    fn clear(&mut self);
    /// Suma `v` en (j, k) con j ≥ k.
    fn add_lower(&mut self, j: usize, k: usize, v: f64);
    /// Suma wgt · row rowᵀ, con la fila dada por índices y valores.
    fn add_row(&mut self, row: &Row, wgt: f64) {
        for p in 0..row.idx.len() {
            let (j, vj) = (row.idx[p], row.val[p] * wgt);
            for q in 0..=p {
                let (k, vk) = (row.idx[q], row.val[q]);
                if j >= k {
                    self.add_lower(j, k, vj * vk);
                } else {
                    self.add_lower(k, j, vj * vk);
                }
            }
        }
    }
    /// Resuelve (A + λ·diag A) x = b. `None` si es singular.
    fn solve_damped(&self, lam: f64, b: &[f64]) -> Option<Vec<f64>>;
    /// Inversa, en la forma que hace falta para la barra de confianza.
    fn covariance(&self) -> Option<Cov>;
}

/// Fila dispersa del jacobiano: posiciones activas y valores.
#[derive(Default)]
struct Row {
    idx: Vec<usize>,
    val: Vec<f64>,
}

impl Row {
    fn clear(&mut self) {
        self.idx.clear();
        self.val.clear();
    }
    fn push(&mut self, j: usize, v: f64) {
        self.idx.push(j);
        self.val.push(v);
    }
}

enum Cov {
    Dense(Vec<Vec<f64>>),
    /// Diagonal y primera subdiagonal de la inversa. Basta cuando cada fila toca anclas vecinas.
    Band { diag: Vec<f64>, off: Vec<f64> },
}

impl Cov {
    fn diag(&self, j: usize) -> f64 {
        match self {
            Cov::Dense(c) => c[j][j],
            Cov::Band { diag, .. } => diag[j],
        }
    }

    /// √(rowᵀ C row).
    fn quad(&self, row: &Row) -> f64 {
        let mut v = 0.0;
        for p in 0..row.idx.len() {
            let (j, rj) = (row.idx[p], row.val[p]);
            for q in 0..row.idx.len() {
                let (k, rk) = (row.idx[q], row.val[q]);
                let c = match self {
                    Cov::Dense(c) => c[j][k],
                    Cov::Band { diag, off } => {
                        if j == k {
                            diag[j]
                        } else if j + 1 == k {
                            off[j]
                        } else if k + 1 == j {
                            off[k]
                        } else {
                            0.0
                        }
                    }
                };
                v += rj * c * rk;
            }
        }
        v.max(0.0).sqrt()
    }
}

struct Dense(Vec<Vec<f64>>);

    impl Dense {
        fn new(n: usize) -> Self {
            Dense(mat(n))
        }
    }

    impl SysMat for Dense {
        fn size(&self) -> usize {
            self.0.len()
        }

        fn clear(&mut self) {
            for r in self.0.iter_mut() {
                r.fill(0.0);
            }
        }

        fn add_lower(&mut self, j: usize, k: usize, v: f64) {
            self.0[j][k] += v;
        }

        fn add_row(&mut self, row: &Row, wgt: f64) {
            let n = row.idx.len();
            if n == self.size() && row.idx.iter().enumerate().all(|(p, &j)| p == j) {
                for jj in 0..n {
                    let rj = row.val[jj];
                    if rj == 0.0 {
                        continue;
                    }
                    let wt = rj * wgt;
                    let line = &mut self.0[jj][..=jj];
                    for (l, v) in line.iter_mut().zip(&row.val[..=jj]) {
                        *l += wt * v;
                    }
                }
                return;
            }
            for p in 0..n {
                let (j, vj) = (row.idx[p], row.val[p] * wgt);
                for q in 0..=p {
                    let (k, vk) = (row.idx[q], row.val[q]);
                    let (r, c) = if j >= k { (j, k) } else { (k, j) };
                    self.0[r][c] += vj * vk;
                }
            }
        }

        fn solve_damped(&self, lam: f64, b: &[f64]) -> Option<Vec<f64>> {
            let n = self.size();
            let mut temp = mat(n);
            for j in 0..n {
                for k in 0..=j {
                    temp[j][k] = self.0[j][k];
                    temp[k][j] = self.0[j][k];
                }
                temp[j][j] = self.0[j][j] * (1.0 + lam);
            }
            let mut x = b.to_vec();
            if gaussj(&mut temp, &mut x) {
                Some(x)
            } else {
                None
            }
        }

        fn covariance(&self) -> Option<Cov> {
            let n = self.size();
            let mut temp = mat(n);
            for j in 0..n {
                for k in 0..=j {
                    temp[j][k] = self.0[j][k];
                    temp[k][j] = self.0[j][k];
                }
            }
            let mut rhs = vec![0.0; n];
            if gaussj(&mut temp, &mut rhs) {
                Some(Cov::Dense(temp))
            } else {
                None
            }
        }
    }

/// Banda simétrica: `rows[j][d]` guarda A[j][j − d]. Con ε₂ cada dato toca dos anclas vecinas
/// y la suavidad une j − 1, j, j + 1, así que la banda es 2 y el paso cuesta O(n).
struct Band {
    rows: Vec<Vec<f64>>,
    bw: usize,
}

impl Band {
    fn new(n: usize) -> Self {
        Band { rows: vec![vec![0.0; 3]; n], bw: 2 }
    }

    fn at(&self, j: usize, d: usize) -> f64 {
        self.rows[j].get(d).copied().unwrap_or(0.0)
    }

    /// L D Lᵀ sin pivotes: la matriz es semidefinida positiva más la diagonal amortiguada.
    fn factor(&self, lam: f64) -> Option<(Vec<Vec<f64>>, Vec<f64>)> {
        let n = self.size();
        let bw = self.bw;
        let mut l = vec![vec![0.0; bw + 1]; n];
        let mut d = vec![0.0; n];
        for j in 0..n {
            let mut dj = self.at(j, 0) * (1.0 + lam);
            for k in j.saturating_sub(bw)..j {
                let ljk = l[j][j - k];
                dj -= ljk * ljk * d[k];
            }
            if !(dj > 1e-300) || !dj.is_finite() {
                return None;
            }
            d[j] = dj;
            for i in j + 1..(j + bw + 1).min(n) {
                let mut v = self.at(i, i - j);
                for k in i.saturating_sub(bw)..j {
                    v -= l[i][i - k] * l[j][j - k] * d[k];
                }
                l[i][i - j] = v / dj;
            }
        }
        Some((l, d))
    }

    fn solve_with(l: &[Vec<f64>], d: &[f64], bw: usize, b: &[f64]) -> Vec<f64> {
        let n = b.len();
        let mut y = b.to_vec();
        for i in 0..n {
            for k in i.saturating_sub(bw)..i {
                y[i] -= l[i][i - k] * y[k];
            }
        }
        for i in 0..n {
            y[i] /= d[i];
        }
        for i in (0..n).rev() {
            for k in i + 1..(i + bw + 1).min(n) {
                y[i] -= l[k][k - i] * y[k];
            }
        }
        y
    }
}

impl SysMat for Band {
    fn size(&self) -> usize {
        self.rows.len()
    }

    fn clear(&mut self) {
        for r in self.rows.iter_mut() {
            r.fill(0.0);
        }
    }

    fn add_lower(&mut self, j: usize, k: usize, v: f64) {
        let d = j - k;
        if d > self.bw {
            self.bw = d;
        }
        let row = &mut self.rows[j];
        if row.len() <= d {
            row.resize(d + 1, 0.0);
        }
        row[d] += v;
    }

    fn solve_damped(&self, lam: f64, b: &[f64]) -> Option<Vec<f64>> {
        let (l, d) = self.factor(lam)?;
        Some(Band::solve_with(&l, &d, self.bw, b))
    }

    fn covariance(&self) -> Option<Cov> {
        let (l, d) = self.factor(0.0)?;
        let n = self.size();
        let mut diag = vec![0.0; n];
        let mut off = vec![0.0; n];
        let mut e = vec![0.0; n];
        for j in 0..n {
            e[j] = 1.0;
            let col = Band::solve_with(&l, &d, self.bw, &e);
            e[j] = 0.0;
            diag[j] = col[j];
            if j + 1 < n {
                off[j] = col[j + 1];
            }
        }
        Some(Cov::Band { diag, off })
    }
}

/// La magnitud medida como función de los coeficientes de ε₂ y de ε∞.
struct Model<'a> {
    measure: Measure,
    /// Para cada dato: la primera ancla que lo toca y el peso de ese triángulo y del siguiente.
    basis: &'a [(usize, f64, f64)],
    kmat: &'a [Vec<f64>],
    e1_fixed: &'a [f64],
    e2_fixed: &'a [f64],
    y: &'a [f64],
    nu: &'a [f64],
    d_cm: f64,
    inv_var: &'a [f64],
    active: &'a [bool],
    active_pos: &'a [Option<usize>],
    /// Número de anclas. El parámetro `ma` es ε∞.
    ma: usize,
    smoothness: f64,
    negativity: f64,
}

impl Model<'_> {
    fn eps(&self, i: usize, p: &[f64]) -> (f64, f64) {
        let (j0, b0, b1) = self.basis[i];
        let e2 = self.e2_fixed[i] + p[j0] * b0 + p[j0 + 1] * b1;
        let mut e1 = p[self.ma] + self.e1_fixed[i];
        if let Some(k) = self.kmat.get(i) {
            for j in 1..self.ma - 1 {
                e1 += p[j] * k[j];
            }
        }
        (e1, e2)
    }

    fn basis_at(&self, i: usize, l: usize) -> f64 {
        let (j0, b0, b1) = self.basis[i];
        if l == j0 {
            b0
        } else if l == j0 + 1 {
            b1
        } else {
            0.0
        }
    }

    fn row(&self, i: usize, d1: f64, d2: f64, row: &mut Row) {
        row.clear();
        match self.kmat.get(i) {
            Some(k) => {
                for (l, pos) in self.active_pos.iter().enumerate() {
                    let Some(jj) = *pos else { continue };
                    row.push(jj, if l == self.ma { d1 } else { d2 * self.basis_at(i, l) + d1 * k[l] });
                }
            }
            None => {
                let (j0, b0, b1) = self.basis[i];
                if let Some(jj) = self.active_pos[j0] {
                    row.push(jj, d2 * b0);
                }
                if let Some(jj) = self.active_pos[j0 + 1] {
                    row.push(jj, d2 * b1);
                }
                if let Some(jj) = self.active_pos[self.ma] {
                    row.push(jj, d1);
                }
            }
        }
    }

    /// Algún punto sigue a más del 2 % del rango de la medida. Ahí el 1 % de χ² no es parada.
    fn still_far(&self, p: &[f64]) -> bool {
        let n = self.y.len();
        if n == 0 {
            return false;
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        let mut worst: f64 = 0.0;
        for i in 0..n {
            let y = self.y[i];
            lo = lo.min(y);
            hi = hi.max(y);
            let (q, _, _, _, _) = self.model(i, p);
            let w = self.inv_var.get(i).copied().unwrap_or(1.0).sqrt();
            worst = worst.max((y - q).abs() * w);
        }
        worst > 0.02 * (hi - lo).max(1e-9)
    }

    fn model(&self, i: usize, p: &[f64]) -> (f64, f64, f64, f64, f64) {
        let (e1, e2) = self.eps(i, p);
        let (q, d1, d2) = quantity_grad(self.measure, e1, e2, self.nu[i], self.d_cm);
        (q, d1, d2, e1, e2)
    }

    /// χ² con suavidad y castigo; α (triángulo inferior) y β al estilo de Numerical Recipes.
    fn mrqcof<M: SysMat>(&self, p: &[f64], out_a: &mut M, out_b: &mut [f64]) -> f64 {
        out_a.clear();
        out_b.fill(0.0);
        let mut row = Row::default();
        let mut chi2 = 0.0;
        for i in 0..self.y.len() {
            let (q, d1, d2, _, e2) = self.model(i, p);
            let dy = self.y[i] - q;
            let wgt = self.inv_var.get(i).copied().unwrap_or(1.0);
            self.row(i, d1, d2, &mut row);
            out_a.add_row(&row, wgt);
            for (p, &jj) in row.idx.iter().enumerate() {
                out_b[jj] += dy * wgt * row.val[p];
            }
            chi2 += dy * dy * wgt;
            if self.negativity > 0.0 && e2 < 0.0 {
                chi2 += self.negativity * e2 * e2;
                self.row(i, 0.0, 1.0, &mut row);
                out_a.add_row(&row, self.negativity);
                for (p, &jj) in row.idx.iter().enumerate() {
                    out_b[jj] += -self.negativity * e2 * row.val[p];
                }
            }
        }
        let ma = self.ma;
        if self.smoothness > 0.0 && ma >= 3 {
            for j in 1..ma - 1 {
                // La cola a cero es solo la extrapolación de Kramers–Kronig.
                // No entra en la suavidad: si contara como vecino 0, empujaría los extremos medidos hacia cero.
                if !self.active[j] || !self.active[j - 1] || !self.active[j + 1] {
                    continue;
                }
                let d2 = p[j - 1] - 2.0 * p[j] + p[j + 1];
                chi2 += self.smoothness * d2 * d2;
                let grad = self.smoothness * d2;
                let pairs = [(j - 1, 1.0), (j, -2.0), (j + 1, 1.0)];
                for a in 0..3 {
                    let (ja, ca) = pairs[a];
                    let Some(pa) = self.active_pos[ja] else { continue };
                    out_b[pa] -= grad * ca;
                    for b in 0..=a {
                        let (jb, cb) = pairs[b];
                        let Some(pb) = self.active_pos[jb] else { continue };
                        let (r, c) = if pa >= pb { (pa, pb) } else { (pb, pa) };
                        out_a.add_lower(r, c, self.smoothness * ca * cb);
                    }
                }
            }
        }
        chi2
    }

    /// Curvatura de los datos solos, sin suavidad ni castigo.
    fn curvature<M: SysMat>(&self, p: &[f64], out: &mut M) {
        out.clear();
        let mut row = Row::default();
        for i in 0..self.y.len() {
            let (_, d1, d2, _, _) = self.model(i, p);
            self.row(i, d1, d2, &mut row);
            out.add_row(&row, self.inv_var[i]);
        }
    }
}

struct LmResult {
    iters_used: usize,
    converged: bool,
    stalled: bool,
}

/// Levenberg–Marquardt (Numerical Recipes §15.5) con la parada de este laboratorio.
fn levenberg<M: SysMat>(
    model: &Model<'_>,
    a: &mut [f64],
    active_pos: &[Option<usize>],
    iter_max: usize,
    floor: f64,
    alpha: &mut M,
    covar: &mut M,
) -> Result<LmResult, String> {
    let mfit = alpha.size();
    let lamda_high: f64 = 10.0;
    let lamda_low: f64 = 0.1;
    let tol: f64 = 0.01;
    let mut beta = vec![0.0; mfit];
    let mut da = vec![0.0; mfit];
    let first = model.mrqcof(a, alpha, &mut beta);
    if !first.is_finite() {
        return Err("el χ² inicial no es finito. Revisa el espectro.".into());
    }
    let mut ochisq = first;
    let mut atry = a.to_vec();
    let mut alamda = 0.001;
    let mut done = 0i32;
    let mut iters_used = 0usize;
    let mut stalled = false;
    for _ in 0..iter_max {
        iters_used += 1;
        if done >= 1 {
            alamda = 0.0;
        }
        let Some(step) = alpha.solve_damped(alamda, &beta) else {
            // El paso con λ = 0 solo cierra el método. Si esa matriz no se invierte, el espectro aceptado sigue valiendo.
            if done >= 1 {
                break;
            }
            if iters_used > 1 && ochisq.is_finite() {
                stalled = true;
                break;
            }
            return Err("La matriz del paso de Levenberg–Marquardt es singular. ω tiene que ser positiva y cubrir un intervalo.".into());
        };
        if done >= 1 {
            break;
        }
        for (l, pos) in active_pos.iter().enumerate() {
            if let Some(jj) = pos {
                atry[l] = a[l] + step[*jj];
            }
        }
        let chisq = model.mrqcof(&atry, covar, &mut da);
        if !chisq.is_finite() {
            stalled = true;
            break;
        }
        if chisq <= ochisq {
            let improvement = ochisq - chisq;
            alamda *= lamda_low;
            ochisq = chisq;
            std::mem::swap(alpha, covar);
            std::mem::swap(&mut beta, &mut da);
            a.copy_from_slice(&atry);
            // Una bajada menor que el 1 % cierra el ajuste solo si ya no queda un punto lejos.
            // Si no, una raya estrecha (el reststrahlen) se abandona cuando la banda ancha ya cuadra.
            if !model.still_far(a) && improvement < floor.max(tol * chisq.abs()) {
                done += 1;
            }
        } else {
            // Un paso que no baja el objetivo, y casi no lo mueve, es la misma parada.
            // Si no, λ crecía hasta 1e8 y un ajuste ya exacto se anunciaba como fallo.
            let flat = (chisq - ochisq) <= floor.max(tol * ochisq.abs());
            alamda *= lamda_high;
            if !model.still_far(a) && flat && alamda >= 1.0 {
                done += 1;
            } else if alamda > 1.0e8 {
                if model.still_far(a) {
                    stalled = true;
                } else {
                    done = done.max(1);
                }
                break;
            }
        }
    }
    Ok(LmResult { iters_used, converged: done >= 1 && !stalled, stalled })
}

fn gaussj(temp: &mut [Vec<f64>], oneda: &mut [f64]) -> bool {
    let n = oneda.len();
    let mut indxc = vec![0usize; n];
    let mut indxr = vec![0usize; n];
    let mut ipiv = vec![0i32; n];
    for i in 0..n {
        let mut big = 0.0;
        let mut irow = 0;
        let mut icol = 0;
        for j in 0..n {
            if ipiv[j] != 1 {
                for k in 0..n {
                    if ipiv[k] == 0 && temp[j][k].abs() >= big {
                        big = temp[j][k].abs();
                        irow = j;
                        icol = k;
                    }
                }
            }
        }
        // Sin pivote usable la eliminación de Gauss–Jordan divide por cero.
        if big < 1e-30 {
            return false;
        }
        ipiv[icol] += 1;
        if irow != icol {
            temp.swap(irow, icol);
            oneda.swap(irow, icol);
        }
        indxr[i] = irow;
        indxc[i] = icol;
        if temp[icol][icol].abs() < 1e-30 {
            return false;
        }
        let piv = 1.0 / temp[icol][icol];
        temp[icol][icol] = 1.0;
        for l in 0..n {
            temp[icol][l] *= piv;
        }
        oneda[icol] *= piv;
        for ll in 0..n {
            if ll == icol {
                continue;
            }
            let dum = temp[ll][icol];
            if dum == 0.0 {
                continue;
            }
            temp[ll][icol] = 0.0;
            for l in 0..n {
                temp[ll][l] -= temp[icol][l] * dum;
            }
            oneda[ll] -= oneda[icol] * dum;
        }
    }
    for l in (0..n).rev() {
        if indxr[l] != indxc[l] {
            for row in temp.iter_mut() {
                row.swap(indxr[l], indxc[l]);
            }
        }
    }
    true
}

/// Σ aⱼ Kⱼ(ω) con un solo g(ω, wₖ) por ancla. Es dkkt_eps1 − 1 en O(ma) logaritmos.
fn kk_sum(om: f64, anchors: &[f64], a: &[f64], g: &mut [f64]) -> f64 {
    let ma = anchors.len();
    for k in 0..ma {
        g[k] = kuzmenko_g(om, anchors[k]);
    }
    let mut sum = 0.0;
    for j in 1..ma.saturating_sub(1) {
        if a[j] == 0.0 {
            continue;
        }
        let (wm, wj, wp) = (anchors[j - 1], anchors[j], anchors[j + 1]);
        let d1 = if (wj - wm).abs() < 1e-30 { 1e-30 } else { wj - wm };
        let d2 = if (wp - wj).abs() < 1e-30 { 1e-30 } else { wp - wj };
        sum += a[j] * (g[j - 1] / d1 - ((wp - wm) * g[j]) / (d1 * d2) + g[j + 1] / d2);
    }
    sum / std::f64::consts::PI
}

fn drude_eps1_part(om: f64, d: Option<&Drude>) -> f64 {
    d.map(|d| eps1_drude(om, d) - d.eps_inf).unwrap_or(0.0)
}

#[cfg(feature = "server")]
const MAX_FIT_POINTS: usize = 10_000;
#[cfg(feature = "server")]
const DENSE_WORK_BUDGET: u64 = 3_000_000_000_000;

pub fn run_fit(input: &FitRequest) -> Result<FitReport, String> {
    let phys = &input.physics;
    let measure = phys.measure;
    let sym = measure.symbol();
    if input.x.len() != input.y.len() || input.x.len() < 3 {
        return Err("ω y el valor deben tener la misma longitud (mínimo 3)".into());
    }
    #[cfg(feature = "server")]
    if input.x.len() > MAX_FIT_POINTS {
        return Err(format!("El ajuste admite hasta {MAX_FIT_POINTS} puntos."));
    }
    #[cfg(feature = "server")]
    if !matches!(measure, Measure::Eps2 | Measure::Lum) {
        let points = (input.x.len() as u64).saturating_add(1);
        let work = points
            .saturating_mul(points)
            .saturating_mul(points)
            .saturating_mul(input.iter_max.max(1) as u64);
        if work > DENSE_WORK_BUDGET {
            return Err("Este ajuste requiere demasiados recursos. Reduce el número de puntos o iteraciones.".into());
        }
    }
    let errors_given = input.errors_given && input.sigma.len() == input.x.len();
    if errors_given {
        for &s in &input.sigma {
            if !s.is_finite() || s <= 0.0 {
                return Err("el error de cada punto tiene que ser positivo".into());
            }
        }
    }
    for (&x, &y) in input.x.iter().zip(input.y.iter()) {
        if !x.is_finite() || x <= 0.0 || !y.is_finite() {
            return Err("ω debe ser finita y positiva, y el valor finito".into());
        }
    }
    if measure.needs_wavenumber() && !phys.unit.known() {
        return Err(format!("{sym} necesita saber en qué está ω: elige cm⁻¹, eV o nm."));
    }
    let d_cm = phys.thickness_um * 1e-4;
    if measure == Measure::T && !(d_cm > 0.0 && d_cm.is_finite()) {
        return Err("La transmitancia necesita el espesor de la lámina, en µm.".into());
    }

    let mut x_in = input.x.clone();
    let mut y_in = input.y.clone();
    let mut s_in = if errors_given { input.sigma.clone() } else { vec![1.0; x_in.len()] };
    if phys.unit == Unit::Nm {
        nm_to_ev(&mut x_in, &mut y_in, &mut s_in, measure == Measure::Lum)?;
    }
    let mut warnings = Vec::new();
    if matches!(measure, Measure::R | Measure::T) {
        let top = y_in.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if top > 1.5 && top <= 100.5 {
            for v in y_in.iter_mut() {
                *v /= 100.0;
            }
            for s in s_in.iter_mut() {
                if errors_given {
                    *s /= 100.0;
                }
            }
            warnings.push(format!("{sym} venía en %: se dividió entre 100."));
        } else if top > 1.5 {
            return Err(format!("{sym} va entre 0 y 1, o en %."));
        }
        if y_in.iter().any(|v| *v < 0.0) {
            return Err(format!("{sym} no puede ser negativa."));
        }
    }
    if matches!(measure, Measure::N | Measure::K | Measure::Alpha) && y_in.iter().any(|v| *v < 0.0) {
        warnings.push(format!("Hay {sym} negativa en los datos. En un medio pasivo no debería pasar."));
    }

    let mut order: Vec<usize> = (0..x_in.len()).collect();
    order.sort_by(|&a, &b| x_in[a].partial_cmp(&x_in[b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut w: Vec<f64> = order.iter().map(|&i| x_in[i]).collect();
    let mut y_data: Vec<f64> = order.iter().map(|&i| y_in[i]).collect();
    let mut sigma: Vec<f64> = order.iter().map(|&i| s_in[i]).collect();
    collapse_duplicates(&mut w, &mut y_data, &mut sigma, errors_given);
    if w.len() < 3 {
        return Err("tras unir frecuencias repetidas quedan menos de 3 puntos".into());
    }
    let ndat = w.len();
    let w_min = w[0];
    let w_max = w[ndat - 1];
    if !(w_max > w_min) || (w_max - w_min) / w_max.max(1e-30) < 1e-6 {
        return Err("ω no tiene rango: hace falta un intervalo de frecuencias".into());
    }
    let freq_ratio = w_max / w_min;
    let log_mesh = input.log_mesh || freq_ratio > 20.0;
    let options = FitOptions {
        log_mesh,
        extended: input.extended,
        smoothness: input.smoothness,
        negativity: input.negativity,
        iter_max: input.iter_max,
        measure,
        unit: phys.unit,
        thickness_um: phys.thickness_um,
        temperature_k: phys.temperature_k,
        eps_inf: phys.eps_inf,
    };

    if measure == Measure::Lum {
        return Ok(luminescence(w, y_data, sigma, errors_given, warnings, options));
    }

    let nu: Vec<f64> = w.iter().map(|&v| phys.unit.wavenumber(v).unwrap_or(0.0)).collect();
    let drude = input.drude.clone();
    let eps2_mod: Vec<f64> = w.iter().map(|&om| drude.as_ref().map(|d| eps2_drude(om, d)).unwrap_or(0.0)).collect();
    let e1_fixed: Vec<f64> = w.iter().map(|&om| drude_eps1_part(om, drude.as_ref())).collect();

    let given = phys.eps_inf.filter(|v| v.is_finite());
    let guess = guess_eps_inf(measure, &y_data);
    let (base, fit_base, eps_inf_note) = match given {
        Some(v) => (v, false, "escrito por ti".to_string()),
        None => match measure {
            Measure::Eps1 | Measure::N => (guess.unwrap_or(1.0), true, "ajustado junto con ε₂".to_string()),
            Measure::R => {
                let v = guess.unwrap_or(1.0);
                warnings.push(format!(
                    "ε∞ vacío: se inicia en {v:.3} a partir de R en el extremo alto y se ajusta junto con ε₂. Con solo R, la solución sigue dependiendo de lo que haya fuera del rango; si conoces ε∞, escríbelo para fijarlo."
                ));
                (v, true, "estimado de R en el extremo alto y ajustado junto con ε₂".to_string())
            }
            Measure::K | Measure::Alpha | Measure::T => {
                warnings.push(format!(
                    "ε∞ vacío: se usa 1. Con {sym} ese 1 no sale de la medida. ε₁, n, R y la pérdida dependen de él."
                ));
                (1.0, false, "1 por defecto".to_string())
            }
            _ => (1.0, false, "1 por defecto".to_string()),
        },
    };
    let auto_log = log_mesh && !input.log_mesh;
    // R usa los puntos medios de los intervalos medidos, como en la malla de la tesis.
    let full_r_mesh = measure == Measure::R;
    let n_anchor = if full_r_mesh { ndat.saturating_sub(2) } else { ndat };
    let mut anchors = if full_r_mesh {
        w.windows(2)
            .take(n_anchor)
            .map(|pair| pair[0] + (pair[1] - pair[0]) * 0.5)
            .collect()
    } else {
        w.clone()
    };
    if full_r_mesh {
        warnings.push(format!(
            "R usa {} anclas interiores en los puntos medios de la malla medida (N−2).",
            anchors.len()
        ));
    }
    // Los extremos medidos quedan libres. Un ancla extra a cada lado, con coeficiente 0,
    // hace que ε₂ baje en línea recta hasta cero. La Kramers–Kronig integra esa cola:
    // si ε₂ se obligara a 0 justo en el primer y el último dato, faltaría peso espectral.
    let span = (w_max - w_min).max(1e-30);
    let ratio = span / w_max.max(1e-30);
    let bump = 0.03_f64.min(0.008_f64.max(0.015 * 1.2_f64.min(ratio)));
    let w_lo = w_min * (1.0 - bump);
    let w_hi = w_max * (1.0 + bump);
    if anchors.first().is_none_or(|v| *v > w_lo * (1.0 + 1e-12)) {
        anchors.insert(0, w_lo);
    }
    if anchors.last().is_none_or(|v| *v < w_hi * (1.0 - 1e-12)) {
        anchors.push(w_hi);
    }
    warnings.push(
        "Fuera del rango medido, ε₂ no queda fijada por los datos. El modelo la lleva linealmente a cero en los nodos de borde; esa extrapolación puede cambiar ε₁ dentro del rango.".into(),
    );
    let ma = anchors.len();
    let mut active = vec![true; ma + 1];
    active[0] = false;
    active[ma - 1] = false;
    active[ma] = fit_base;
    let mfit = active.iter().filter(|a| **a).count();
    if mfit < 1 {
        return Err("hacen falta al menos 3 anclas".into());
    }
    let active_pos: Vec<Option<usize>> = {
        let mut p = 0;
        active
            .iter()
            .map(|on| {
                if *on {
                    p += 1;
                    Some(p - 1)
                } else {
                    None
                }
            })
            .collect()
    };

    let seed: Vec<f64> = seed_eps2(measure, &w, &y_data, &nu, d_cm, base)
        .iter()
        .zip(&eps2_mod)
        .map(|(s, m)| s - m)
        .collect();
    let mut a = vec![0.0; ma + 1];
    for j in 1..ma - 1 {
        a[j] = interp(anchors[j], &w, &seed);
    }
    a[ma] = base;

    // Cada dato cae entre dos anclas consecutivas: solo esos dos triángulos no son cero.
    let mut basis: Vec<(usize, f64, f64)> = Vec::with_capacity(ndat);
    let mut seg = 0usize;
    for &om in &w {
        while seg + 2 < ma && anchors[seg + 1] <= om {
            seg += 1;
        }
        basis.push((seg, tri_basis(om, seg, &anchors), tri_basis(om, seg + 1, &anchors)));
    }
    let kmat: Vec<Vec<f64>> = if measure == Measure::Eps2 {
        Vec::new()
    } else {
        w.iter()
            .map(|&om| {
                let mut row = vec![0.0; ma];
                for j in 1..ma - 1 {
                    row[j] = kk_kernel(om, anchors[j - 1], anchors[j], anchors[j + 1]);
                }
                row
            })
            .collect()
    };

    let smoothness = input.smoothness.max(0.0);
    let negativity = input.negativity.max(0.0);
    let inv_var: Vec<f64> = sigma.iter().map(|s| 1.0 / (s * s)).collect();
    let model = Model {
        measure,
        basis: &basis,
        kmat: &kmat,
        e1_fixed: &e1_fixed,
        e2_fixed: &eps2_mod,
        y: &y_data,
        nu: &nu,
        d_cm,
        inv_var: &inv_var,
        active: &active,
        active_pos: &active_pos,
        ma,
        smoothness,
        negativity,
    };

    let iter_max = input.iter_max.max(1);
    // Piso de la parada en la escala de los datos: T y R viven entre 0 y 1, ε₂ puede valer cientos.
    let floor = 1e-12 * y_data.iter().zip(&inv_var).map(|(y, w)| y * y * w).sum::<f64>().max(1e-300);

    // Con ε₂ nada acopla anclas lejanas: la matriz es de banda y el paso cuesta O(n) en vez de O(n³).
    let banded = measure == Measure::Eps2 && !fit_base;
    let (lm, cov) = if banded {
        let mut alpha = Band::new(mfit);
        let mut covar = Band::new(mfit);
        let lm = levenberg(&model, &mut a, &active_pos, iter_max, floor, &mut alpha, &mut covar)?;
        let cov = if errors_given {
            model.curvature(&a, &mut alpha);
            alpha.covariance()
        } else {
            None
        };
        (lm, cov)
    } else {
        let mut alpha = Dense::new(mfit);
        let mut covar = Dense::new(mfit);
        let lm = levenberg(&model, &mut a, &active_pos, iter_max, floor, &mut alpha, &mut covar)?;
        let cov = if errors_given {
            model.curvature(&a, &mut alpha);
            alpha.covariance()
        } else {
            None
        };
        (lm, cov)
    };
    let LmResult { iters_used, converged, stalled } = lm;
    let base = a[ma];

    let unit = phys.unit;
    let n_inner = ma - 2;
    let mut omega = Vec::with_capacity(n_inner);
    let mut eps1 = Vec::with_capacity(n_inner);
    let mut eps2 = Vec::with_capacity(n_inner);
    let mut n_index = Vec::with_capacity(n_inner);
    let mut k_index = Vec::with_capacity(n_inner);
    let mut reflectivity = Vec::with_capacity(n_inner);
    let mut loss = Vec::with_capacity(n_inner);
    let mut sigma1 = Vec::new();
    let mut sigma2 = Vec::new();
    let mut absorption = Vec::new();
    let mut transmittance = Vec::new();
    let mut absorbance = Vec::new();
    let mut gbuf = vec![0.0; ma];
    for i in 0..n_inner {
        let om = anchors[i + 1];
        let e2 = a[i + 1] + drude.as_ref().map(|d| eps2_drude(om, d)).unwrap_or(0.0);
        let e1 = base + drude_eps1_part(om, drude.as_ref()) + kk_sum(om, &anchors, &a[..ma], &mut gbuf);
        let (ni, ki, r) = optics(e1, e2);
        omega.push(om);
        eps1.push(e1);
        eps2.push(e2);
        n_index.push(ni);
        k_index.push(ki);
        reflectivity.push(r);
        let mag2 = e1 * e1 + e2 * e2;
        loss.push(if mag2 > 1e-300 { e2 / mag2 } else { 0.0 });
        if let Some(v) = unit.wavenumber(om) {
            sigma1.push(v * e2 / 60.0);
            sigma2.push(v * (base - e1) / 60.0);
            absorption.push(4.0 * std::f64::consts::PI * v * ki);
            if d_cm > 0.0 {
                let t = quantity(Measure::T, e1, e2, v, d_cm).clamp(0.0, 1.0);
                transmittance.push(t);
                absorbance.push(if t > 1e-12 { -t.log10() } else { 12.0 });
            }
        }
    }

    let mut ext_omega = Vec::new();
    let mut ext_eps1 = Vec::new();
    let mut ext_eps2 = Vec::new();
    if input.extended {
        let lo = (w_min * 0.7).max(w_min * 1e-6);
        let hi = w_max * 1.4;
        for om in logspace(lo, hi, 96) {
            let e2_var = (1..ma - 1).map(|j| a[j] * tri_basis(om, j, &anchors)).sum::<f64>();
            let e2_m = drude.as_ref().map(|d| eps2_drude(om, d)).unwrap_or(0.0);
            let e1 = base + drude_eps1_part(om, drude.as_ref()) + kk_sum(om, &anchors, &a[..ma], &mut gbuf);
            ext_omega.push(om);
            ext_eps1.push(e1);
            ext_eps2.push(e2_var + e2_m);
        }
    }

    let neg = eps2.iter().filter(|v| **v < 0.0).count();
    if neg > 2.max(eps2.len() / 20) {
        warnings.push(format!("ε₂ es negativa en {neg}/{} puntos interiores.", eps2.len()));
    }
    let dof_n = ndat.saturating_sub(mfit);
    let mut data_chisq = 0.0;
    let mut raw_sq = 0.0;
    let mut model_q = Vec::with_capacity(ndat);
    let mut model_eps2 = Vec::with_capacity(ndat);
    let mut difference = Vec::with_capacity(ndat);
    for i in 0..ndat {
        let (q, _, _, _, e2) = model.model(i, &a);
        let dy = y_data[i] - q;
        data_chisq += dy * dy * inv_var[i];
        raw_sq += dy * dy;
        model_q.push(q);
        model_eps2.push(e2);
        difference.push(dy * -100.0);
    }
    let chisq_dof = if dof_n == 0 { data_chisq } else { data_chisq / dof_n as f64 };
    let rms = (raw_sq / ndat as f64).sqrt();
    let scale = y_data.iter().fold(0.0, |s, v| s + v.abs()) / ndat as f64;
    if scale > 1e-12 && rms > 0.5 * scale {
        warnings.push(format!(
            "El error típico de {sym} es {rms:.3}, más de la mitad del |{sym}| medio ({scale:.3}). El modelo queda lejos de los datos."
        ));
    }
    if auto_log {
        warnings.push(format!(
            "Eje ω en escala log: el rango es ×{freq_ratio:.0}. El laboratorio Qwik usa ese corte a partir de ×20."
        ));
    }
    let below = model_eps2.iter().filter(|v| **v < -1e-9).count();
    let data_below = measure == Measure::Eps2 && y_data.iter().any(|v| *v < 0.0);
    if data_below && negativity > 0.0 {
        let below = y_data.iter().filter(|v| **v < 0.0).count();
        warnings.push(format!(
            "{below} datos están bajo cero. El castigo de ε₂ negativa ({negativity:.3}) no deja que el ajuste baje ahí. Si el valle es real, pon ese control en 0. Más pasos no lo cruzan."
        ));
    } else if below > 0 && negativity == 0.0 && measure != Measure::Eps2 {
        warnings.push(format!("ε₂ quedó negativa en {below} datos. Sube el castigo de ε₂ negativa para un medio pasivo."));
    }
    match measure {
        Measure::Eps1 => warnings.push(
            "Con solo ε₁, ε₂ queda poco determinada: lo que pase fuera del rango cambia ε₁ igual que una banda dentro.".into(),
        ),
        Measure::T => {
            let dark = y_data.iter().filter(|v| **v < 1e-3).count();
            if dark > 0 {
                warnings.push(format!(
                    "{dark} datos tienen T < 0.001: ahí la lámina es opaca y k solo queda acotada por abajo. Una lámina más fina lo resuelve."
                ));
            }
        }
        _ => {}
    }
    if stalled {
        warnings.push(
            "El paso se frenó (λ demasiado grande o un χ² no finito). En pantalla está el mejor espectro aceptado.".into(),
        );
    } else if !converged && iters_used >= iter_max {
        warnings.push(format!(
            "Llegó a los {iter_max} pasos y algún punto sigue lejos. En pantalla está el mejor espectro aceptado."
        ));
    }

    // Barra de confianza: inversa de la curvatura de los datos, sin suavidad ni castigo.
    // Es la covarianza del mínimo cuadrado pesado (trabajo de grado, ec. 2.123, con δχ² = 1).
    let mut q_err = Vec::new();
    let mut eps_inf_err = None;
    let (point_err, coef_err) = match cov {
        Some(cov) => {
            let mut row = Row::default();
            let mut point_err = Vec::with_capacity(ndat);
            for i in 0..ndat {
                model.row(i, 0.0, 1.0, &mut row);
                point_err.push(cov.quad(&row));
                if banded {
                    q_err.push(*point_err.last().unwrap());
                } else {
                    let (_, d1, d2, _, _) = model.model(i, &a);
                    model.row(i, d1, d2, &mut row);
                    q_err.push(cov.quad(&row));
                }
            }
            let mut coef_err = vec![0.0; n_inner];
            for i in 0..n_inner {
                if let Some(jj) = active_pos[i + 1] {
                    coef_err[i] = cov.diag(jj).max(0.0).sqrt();
                }
            }
            if let Some(jj) = active_pos[ma] {
                eps_inf_err = Some(cov.diag(jj).max(0.0).sqrt());
            }
            (point_err, coef_err)
        }
        None if errors_given => {
            warnings.push("No salió la barra de confianza: la curvatura de los datos es singular.".into());
            (Vec::new(), Vec::new())
        }
        None => (Vec::new(), Vec::new()),
    };

    let peaks = find_peaks(&omega, &eps2);
    if let Some(w) = index_background_warning(measure, base, &n_index, &k_index) {
        warnings.push(w);
    }
    if let Some((imax, ymax)) = high_end_maximum(&eps2) {
        let top = peaks.iter().max_by(|a, b| a.height.partial_cmp(&b.height).unwrap_or(std::cmp::Ordering::Equal));
        let span = omega.last().copied().unwrap_or(0.0) - omega.first().copied().unwrap_or(0.0);
        let separated = top.is_some_and(|p| (omega[imax] - p.position).abs() > 0.05 * span.abs());
        if let Some(p) = top.filter(|p| separated && ymax > 1.8 * p.height.max(1e-12)) {
            warnings.push(format!(
                "ε₂ llega a {ymax:.4} al final del rango y no baja: es el borde de absorción, cortado por la ventana. El pico de la tabla ({:.4}, altura {:.4}) es un máximo local más bajo.",
                p.position, p.height
            ));
        } else if top.is_none() {
            warnings.push(format!(
                "ε₂ llega a {ymax:.4} al final del rango y no cierra un pico. Es un borde de absorción abierto por la ventana."
            ));
        }
    }
    let energies: Option<Vec<f64>> = omega.iter().map(|&om| unit.energy(om)).collect();
    let rising = absorption.last().is_some_and(|last| {
        absorption.iter().copied().fold(f64::NEG_INFINITY, f64::max) > 0.0
            && *last >= 0.98 * absorption.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    });
    let (direct_edge, indirect_edge) = match (&energies, absorption.len() == omega.len()) {
        (Some(e), true) => (tauc_edge(e, &absorption, 2.0), tauc_edge(e, &absorption, 0.5)),
        _ => (None, None),
    };
    let (tauc_show, tauc_note) = tauc_account(direct_edge, indirect_edge, rising);
    let tauc_show = tauc_show.to_string();
    if !tauc_note.is_empty() {
        warnings.push(tauc_note.clone());
    }
    let tauc_direct = direct_edge.map(|e| e.gap);
    let tauc_indirect = indirect_edge.map(|e| e.gap);
    let tauc_direct_r2 = direct_edge.map(|e| e.r2);
    let tauc_indirect_r2 = indirect_edge.map(|e| e.r2);
    let drude = drude.map(|mut d| {
        d.eps_inf = base;
        d
    });

    Ok(FitReport {
        omega,
        eps1,
        eps2,
        data_x: w,
        data_y: y_data,
        n_index,
        k_index,
        reflectivity,
        ext_omega,
        ext_eps1,
        ext_eps2,
        chisq: data_chisq,
        chisq_dof,
        n_anchor: ma,
        mfit,
        points_used: ndat,
        warnings,
        drude,
        iters_used,
        converged,
        model_eps2,
        difference,
        options,
        data_sigma: if errors_given { sigma } else { Vec::new() },
        eps2_err: point_err,
        coef_err,
        errors_given,
        model_q,
        q_err,
        loss,
        sigma1,
        sigma2,
        absorption,
        transmittance,
        absorbance,
        peaks,
        eps_inf: Some(base),
        eps_inf_err,
        eps_inf_note,
        tauc_direct,
        tauc_indirect,
        tauc_direct_r2,
        tauc_indirect_r2,
        tauc_show,
        tauc_note,
        ..FitReport::empty()
    })
}

/// Luminiscencia: no hay Kramers–Kronig. Picos, integral, centroide, temperatura
/// de la cola alta y la forma de la absorción por van Roosbroeck–Shockley.
fn luminescence(
    w: Vec<f64>,
    y: Vec<f64>,
    sigma: Vec<f64>,
    errors_given: bool,
    mut warnings: Vec<String>,
    options: FitOptions,
) -> FitReport {
    let unit = options.unit;
    let peaks = find_peaks(&w, &y);
    let (integral, centroid) = integral_and_centroid(&w, &y);
    let energies: Option<Vec<f64>> = w.iter().map(|&v| unit.energy(v)).collect();
    let (mut vrs_x, mut vrs_y, mut carrier_temp) = (Vec::new(), Vec::new(), None);
    match &energies {
        Some(e) => {
            carrier_temp = tail_temperature(e, &y);
            if options.temperature_k > 0.0 {
                let (ex, ay) = roosbroeck_shockley(e, &y, options.temperature_k);
                vrs_y = ay;
                vrs_x = ex
                    .iter()
                    .map(|&en| if unit == Unit::Cm { en / EV_PER_CM } else { en })
                    .collect();
            }
        }
        None => warnings.push(
            "Con ω sin unidad salen picos, anchos, áreas y el centroide. Elige cm⁻¹, eV o nm para la temperatura de la cola y la absorción por van Roosbroeck–Shockley.".into(),
        ),
    }
    if peaks.is_empty() {
        warnings.push("No hay picos con prominencia de al menos el 5 % del rango.".into());
    }
    if y.iter().any(|v| *v < 0.0) {
        warnings.push("Hay intensidad negativa: suele ser un fondo restado de más.".into());
    }
    let ndat = w.len();
    FitReport {
        data_x: w,
        data_y: y,
        points_used: ndat,
        warnings,
        converged: true,
        options,
        data_sigma: if errors_given { sigma } else { Vec::new() },
        errors_given,
        peaks,
        integral: Some(integral),
        centroid,
        carrier_temp,
        vrs_x,
        vrs_y,
        ..FitReport::empty()
    }
}

/// Con α, k o T y ε∞ = 1, el extremo transparente deja n pegado a 1.
fn index_background_warning(measure: Measure, eps_inf: f64, n: &[f64], k: &[f64]) -> Option<String> {
    if !matches!(measure, Measure::Alpha | Measure::K | Measure::T) || (eps_inf - 1.0).abs() > 0.05 {
        return None;
    }
    let npts = n.len().min(k.len());
    if npts < 5 {
        return None;
    }
    let take = (npts / 8).clamp(4, 24);
    let mean = |xs: &[f64]| xs.iter().take(take).map(|v| v.abs()).sum::<f64>() / take as f64;
    let nk = mean(k);
    let nn = mean(n);
    if nk < 0.05 && nn < 1.35 {
        Some(format!(
            "En el extremo de baja energía la muestra es casi transparente (k ≈ {nk:.3}) y n sale {nn:.2}. α y k no fijan el índice: con ε∞ = 1, n se queda cerca de 1, y ε₁, R y la pérdida salen bajas. Si en esa zona conoces n, escribe ε∞ = n² y vuelve a analizar. Esa constante es la de encima de tu rango."
        ))
    } else {
        None
    }
}

fn collapse_duplicates(w: &mut Vec<f64>, y: &mut Vec<f64>, sigma: &mut Vec<f64>, errors_given: bool) {
    if w.is_empty() {
        return;
    }
    let mut out_w = Vec::with_capacity(w.len());
    let mut out_y = Vec::with_capacity(y.len());
    let mut out_s = Vec::with_capacity(sigma.len());
    let mut i = 0;
    while i < w.len() {
        let mut j = i + 1;
        let tol = w[i].abs() * 1e-9 + 1e-12;
        while j < w.len() && (w[j] - w[i]).abs() <= tol {
            j += 1;
        }
        if errors_given {
            let mut num = 0.0;
            let mut den = 0.0;
            for k in i..j {
                let wgt = 1.0 / (sigma[k] * sigma[k]);
                num += y[k] * wgt;
                den += wgt;
            }
            out_y.push(if den > 0.0 { num / den } else { y[i] });
            out_s.push(if den > 0.0 { 1.0 / den.sqrt() } else { sigma[i] });
        } else {
            let mut acc = 0.0;
            for k in i..j {
                acc += y[k];
            }
            out_y.push(acc / (j - i) as f64);
            out_s.push(1.0);
        }
        out_w.push(w[i]);
        i = j;
    }
    *w = out_w;
    *y = out_y;
    *sigma = out_s;
}

#[derive(Clone, Debug, Default)]
pub struct TextFit {
    pub iter_max: usize,
    pub log_mesh: bool,
    pub extended: bool,
    pub drude_on: bool,
    pub eps_inf: String,
    pub omega_p: String,
    pub gamma: String,
    /// Peso de la segunda diferencia de los coeficientes. 0 = no suaviza.
    pub smoothness: f64,
    /// Peso de ε₂ negativa. 0 = se permite. En un medio pasivo ε₂ debería ser ≥ 0.
    pub negativity: f64,
    pub measure: String,
    pub unit: String,
    /// Espesor en µm, para T.
    pub thickness: String,
    /// Temperatura en K, para la luminiscencia.
    pub temperature: String,
}

pub fn fit_text(text: &str, opts: &TextFit) -> Result<FitReport, String> {
    let (x, y, sigma, errors_given) = parse_spectrum(text)?;
    let eps_inf = parse_maybe(&opts.eps_inf);
    let measure = Measure::parse(&opts.measure);
    let drude = if opts.drude_on && measure.uses_kk() {
        let auto = suggest_drude(&x, &y);
        Some(Drude {
            eps_inf: eps_inf.unwrap_or(1.0),
            omega_p: parse_opt(&opts.omega_p, auto.omega_p).max(0.0),
            gamma: parse_opt(&opts.gamma, auto.gamma).max(1e-9),
        })
    } else {
        None
    };
    run_fit(&FitRequest {
        x,
        y,
        iter_max: opts.iter_max.clamp(1, 1000),
        log_mesh: opts.log_mesh,
        drude,
        smoothness: finite_nonneg(opts.smoothness, 0.08),
        negativity: finite_nonneg(opts.negativity, 100.0),
        extended: opts.extended,
        sigma,
        errors_given,
        physics: Physics {
            measure,
            unit: Unit::parse(&opts.unit),
            thickness_um: parse_opt(&opts.thickness, 0.0).max(0.0),
            temperature_k: parse_opt(&opts.temperature, 300.0).max(0.0),
            eps_inf,
        },
    })
}

fn parse_maybe(raw: &str) -> Option<f64> {
    raw.trim().replace(',', ".").parse::<f64>().ok().filter(|v| v.is_finite())
}

fn finite_nonneg(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value >= 0.0 {
        value.min(1.0e6)
    } else {
        fallback
    }
}

fn parse_opt(raw: &str, fallback: f64) -> f64 {
    parse_maybe(raw).unwrap_or(fallback)
}

/// Tabla principal en CSV con cabecera: ε en cada ancla, o la intensidad en luminiscencia.
pub fn report_csv(report: &FitReport) -> String {
    let mut out = String::new();
    let x_name = report.options.unit.axis().replace(' ', "_");
    if report.measure() == Measure::Lum {
        let with_vrs = report.vrs_x.len() == report.data_x.len();
        out.push_str(&format!("{x_name},I{}\n", if with_vrs { ",alpha_rel" } else { "" }));
        for i in 0..report.data_x.len() {
            out.push_str(&format!("{},{}", report.data_x[i], report.data_y[i]));
            if with_vrs {
                out.push_str(&format!(",{}", report.vrs_y[i]));
            }
            out.push('\n');
        }
        return out;
    }
    let n = report.omega.len();
    let with_err = report.coef_err.len() == n;
    let with_units = report.sigma1.len() == n && report.absorption.len() == n;
    let with_loss = report.loss.len() == n;
    let mut head = format!("{x_name},eps1,eps2,n,k,R");
    if with_loss {
        head.push_str(",loss");
    }
    if with_units {
        head.push_str(",sigma1_ohm-1cm-1,sigma2_ohm-1cm-1,alpha_cm-1");
    }
    let with_film = report.transmittance.len() == n && report.absorbance.len() == n;
    if with_film {
        head.push_str(",T,A");
    }
    if with_err {
        head.push_str(",s_eps2");
    }
    out.push_str(&head);
    out.push('\n');
    for i in 0..n {
        out.push_str(&format!(
            "{},{},{},{},{},{}",
            report.omega[i], report.eps1[i], report.eps2[i], report.n_index[i], report.k_index[i], report.reflectivity[i]
        ));
        if with_loss {
            out.push_str(&format!(",{}", report.loss[i]));
        }
        if with_units {
            out.push_str(&format!(",{},{},{}", report.sigma1[i], report.sigma2[i], report.absorption[i]));
        }
        if with_film {
            out.push_str(&format!(",{},{}", report.transmittance[i], report.absorbance[i]));
        }
        if with_err {
            out.push_str(&format!(",{}", report.coef_err[i]));
        }
        out.push('\n');
    }
    out
}

/// Datos contra modelo, punto a punto.
pub fn fit_csv(report: &FitReport) -> String {
    let x_name = report.options.unit.axis().replace(' ', "_");
    let sym = report.measure().key();
    let n = report.data_x.len();
    let model: &[f64] = if report.model_q.len() == n {
        &report.model_q
    } else if report.model_eps2.len() == n {
        &report.model_eps2
    } else {
        &[]
    };
    let with_sigma = report.data_sigma.len() == n;
    let with_err = report.q_err.len() == n;
    let mut out = format!("{x_name},{sym}_dato");
    if with_sigma {
        out.push_str(",error");
    }
    if !model.is_empty() {
        out.push_str(&format!(",{sym}_modelo,diferencia"));
    }
    if with_err {
        out.push_str(",s_modelo");
    }
    out.push('\n');
    for i in 0..n {
        out.push_str(&format!("{},{}", report.data_x[i], report.data_y[i]));
        if with_sigma {
            out.push_str(&format!(",{}", report.data_sigma[i]));
        }
        if !model.is_empty() {
            out.push_str(&format!(",{},{}", model[i], model[i] - report.data_y[i]));
        }
        if with_err {
            out.push_str(&format!(",{}", report.q_err[i]));
        }
        out.push('\n');
    }
    out
}

/// Tabla de picos en CSV.
pub fn peaks_csv(report: &FitReport) -> String {
    let x_name = report.options.unit.axis().replace(' ', "_");
    let mut out = format!("pico,{x_name},altura,prominencia,fwhm,area\n");
    for (i, p) in report.peaks.iter().enumerate() {
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            i + 1,
            p.position,
            p.height,
            p.prominence,
            p.fwhm.map(|v| v.to_string()).unwrap_or_default(),
            p.area
        ));
    }
    out
}

/// Todo el informe, tal cual se guarda.
pub fn report_json(report: &FitReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into())
}

/// Un archivo de descarga por formato: (contenido, tipo MIME, extensión).
pub fn export(report: &FitReport, format: &str, source: &str) -> Option<(String, &'static str, &'static str)> {
    match format {
        "dat" => Some((report_dat(report), "text/plain; charset=utf-8", "dat")),
        "csv" => Some((report_csv(report), "text/csv; charset=utf-8", "csv")),
        "ajuste" => Some((fit_csv(report), "text/csv; charset=utf-8", "ajuste.csv")),
        "picos" if !report.peaks.is_empty() => Some((peaks_csv(report), "text/csv; charset=utf-8", "picos.csv")),
        "json" => Some((report_json(report), "application/json; charset=utf-8", "json")),
        "informe" => Some((crate::informe::html(report, source), "text/html; charset=utf-8", "html")),
        _ => None,
    }
}

pub fn report_dat(report: &FitReport) -> String {
    let measure = report.measure();
    if measure == Measure::Lum {
        let mut out = String::from("# Lightmatter — luminiscencia: picos y forma\n");
        out.push_str("# pico  posicion  altura  prominencia  fwhm  area\n");
        for (i, p) in report.peaks.iter().enumerate() {
            out.push_str(&format!(
                "# {}  {:.8}  {:.8}  {:.8}  {}  {:.8}\n",
                i + 1,
                p.position,
                p.height,
                p.prominence,
                p.fwhm.map(|v| format!("{v:.8}")).unwrap_or_else(|| "-".into()),
                p.area
            ));
        }
        if let Some(c) = report.centroid {
            out.push_str(&format!("# centroide = {c:.8}  integral = {:.8}\n", report.integral.unwrap_or(0.0)));
        }
        if let Some(t) = report.carrier_temp {
            out.push_str(&format!("# T cola alta = {t:.2} K\n"));
        }
        let with_vrs = report.vrs_x.len() == report.data_x.len();
        out.push_str(if with_vrs { "# x  I  alpha_rel\n" } else { "# x  I\n" });
        for i in 0..report.data_x.len() {
            out.push_str(&format!("{:.8}  {:.8}", report.data_x[i], report.data_y[i]));
            if with_vrs {
                out.push_str(&format!("  {:.8}", report.vrs_y[i]));
            }
            out.push('\n');
        }
        return out;
    }
    let mut out = String::from("# Lightmatter — Kramers-Kronig variacional (Kuzmenko 2005)\n");
    out.push_str(&format!(
        "# medida = {}  unidad = {}  eps_inf = {} ({})\n",
        measure.key(),
        report.options.unit.key(),
        report.eps_inf.map(|v| format!("{v:.6}")).unwrap_or_else(|| "1".into()),
        report.eps_inf_note
    ));
    out.push_str(&format!(
        "# chi2 = {}  chi2/dof = {:.6}  anclas = {}  coeficientes = {}  puntos = {}\n",
        report.chisq, report.chisq_dof, report.n_anchor, report.mfit, report.points_used
    ));
    for p in &report.peaks {
        out.push_str(&format!(
            "# pico eps2  {:.8}  altura {:.8}  fwhm {}\n",
            p.position,
            p.height,
            p.fwhm.map(|v| format!("{v:.8}")).unwrap_or_else(|| "-".into())
        ));
    }
    if let Some(g) = report.tauc_direct {
        let r2 = report.tauc_direct_r2.map(|v| format!("  R2 = {v:.2}")).unwrap_or_default();
        out.push_str(&format!("# Tauc directa = {g:.6} eV{r2}\n"));
    }
    if let Some(g) = report.tauc_indirect {
        let r2 = report.tauc_indirect_r2.map(|v| format!("  R2 = {v:.2}")).unwrap_or_default();
        out.push_str(&format!("# Tauc indirecta = {g:.6} eV{r2}\n"));
    }
    if !report.tauc_note.is_empty() {
        out.push_str(&format!("# {}\n", report.tauc_note));
    }
    for w in &report.warnings {
        if w == &report.tauc_note {
            continue;
        }
        out.push_str(&format!("# {w}\n"));
    }
    let n = report.omega.len();
    let with_err = report.coef_err.len() == n;
    let with_units = report.sigma1.len() == n && report.absorption.len() == n;
    let with_loss = report.loss.len() == n;
    let mut head = String::from("# omega  eps1  eps2  n  k  R");
    if with_loss {
        head.push_str("  loss");
    }
    if with_units {
        head.push_str("  sigma1_ohm-1cm-1  sigma2_ohm-1cm-1  alpha_cm-1");
    }
    let with_film = report.transmittance.len() == n && report.absorbance.len() == n;
    if with_film {
        head.push_str("  T  A");
    }
    if with_err {
        head.push_str("  s_eps2");
    }
    out.push_str(&head);
    out.push('\n');
    for i in 0..n {
        out.push_str(&format!(
            "{:.8}  {:.8}  {:.8}  {:.8}  {:.8}  {:.8}",
            report.omega[i], report.eps1[i], report.eps2[i], report.n_index[i], report.k_index[i], report.reflectivity[i]
        ));
        if with_loss {
            out.push_str(&format!("  {:.8}", report.loss[i]));
        }
        if with_units {
            out.push_str(&format!("  {:.8}  {:.8}  {:.8}", report.sigma1[i], report.sigma2[i], report.absorption[i]));
        }
        if with_film {
            out.push_str(&format!("  {:.8}  {:.8}", report.transmittance[i], report.absorbance[i]));
        }
        if with_err {
            out.push_str(&format!("  {:.8}", report.coef_err[i]));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::quantity;

    /// Dos osciladores de Lorentz sobre ε∞ = 2.5, ω en cm⁻¹.
    fn lorentz(w: f64) -> (f64, f64) {
        let mut re = 2.5;
        let mut im = 0.0;
        for (w0, wp, ga) in [(300.0, 400.0, 30.0), (600.0, 300.0, 60.0)] {
            let dr: f64 = w0 * w0 - w * w;
            let di = ga * w;
            let den = dr * dr + di * di;
            re += wp * wp * dr / den;
            im += wp * wp * di / den;
        }
        (re, im)
    }

    /// Ajusta la medida sintética y devuelve |Δε₂| medio en el interior.
    fn recover(measure: Measure, eps_inf: Option<f64>, d_um: f64) -> (FitReport, f64) {
        let w: Vec<f64> = (0..120).map(|i| 100.0 + 900.0 * i as f64 / 119.0).collect();
        let y: Vec<f64> = w
            .iter()
            .map(|&om| {
                let (e1, e2) = lorentz(om);
                quantity(measure, e1, e2, om, d_um * 1e-4)
            })
            .collect();
        let report = run_fit(&FitRequest {
            x: w,
            y,
            iter_max: 80,
            log_mesh: false,
            drude: None,
            smoothness: 0.0,
            negativity: 100.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics { measure, unit: Unit::Cm, thickness_um: d_um, temperature_k: 300.0, eps_inf },
        })
        .unwrap();
        let mut err = 0.0;
        let mut count = 0.0;
        for (om, e2) in report.omega.iter().zip(&report.eps2).skip(10).take(100) {
            err += (e2 - lorentz(*om).1).abs();
            count += 1.0;
        }
        (report, err / count)
    }

    #[test]
    fn every_kk_measure_gives_back_eps2() {
        for (m, eps_inf, d, limit) in [
            (Measure::K, Some(2.5), 0.0, 0.03),
            (Measure::Alpha, Some(2.5), 0.0, 0.03),
            (Measure::T, Some(2.5), 2.0, 0.05),
            (Measure::R, Some(2.5), 0.0, 0.05),
            (Measure::R, None, 0.0, 0.05),
            (Measure::N, None, 0.0, 0.15),
        ] {
            let (report, err) = recover(m, eps_inf, d);
            assert_eq!(report.model_q.len(), report.data_x.len());
            assert!(
                report.warnings.iter().any(|w| w.contains("ε₂ no queda fijada por los datos")),
                "{m:?}: {:?}", report.warnings
            );
            if m == Measure::R {
                assert!(report.chisq < 1e-3, "R χ² {} pasos {}", report.chisq, report.iters_used);
                assert_eq!(report.n_anchor, report.points_used);
                let expected_mfit = report.points_used - if eps_inf.is_some() { 2 } else { 1 };
                assert_eq!(report.mfit, expected_mfit);
                let first_interval = report.data_x[1] - report.data_x[0];
                assert!((report.omega[0] - (report.data_x[0] + first_interval * 0.5)).abs() < 1e-9);
            } else {
                assert!(err < limit, "{m:?}: |Δε₂| {err} pasos {} χ² {}", report.iters_used, report.chisq);
            }
        }
    }

    #[test]
    fn narrow_triangular_r_is_fitted_by_the_variational_mesh() {
        let x: Vec<f64> = (0..41).map(|i| 120.0 + 1.75 * i as f64).collect();
        let eps2: Vec<f64> = x
            .iter()
            .map(|&om| 3.0 * (1.0 - (om - 155.0).abs() / 7.0).max(0.0))
            .collect();
        let y: Vec<f64> = x
            .iter()
            .zip(&eps2)
            .map(|(&om, &e2)| {
                let e1 = 2.5 + dkkt_eps1(om, &x, &eps2) - 1.0;
                quantity(Measure::R, e1, e2, om, 0.0)
            })
            .collect();
        let report = run_fit(&FitRequest {
            x,
            y: y.clone(),
            iter_max: 240,
            log_mesh: false,
            drude: None,
            smoothness: 0.0,
            negativity: 100.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics {
                measure: Measure::R,
                unit: Unit::Cm,
                eps_inf: Some(2.5),
                ..Physics::default()
            },
        })
        .unwrap();
        let rms = (report
            .model_q
            .iter()
            .zip(&y)
            .map(|(fit, data)| (fit - data).powi(2))
            .sum::<f64>()
            / report.points_used as f64)
            .sqrt();
        assert!(rms < 0.02, "R RMS {rms}, χ² {}", report.chisq);
    }

    #[test]
    fn transparent_end_asks_for_n_squared() {
        let n = vec![1.05; 40];
        let k = vec![0.001; 40];
        let w = index_background_warning(Measure::Alpha, 1.0, &n, &k).unwrap();
        assert!(w.contains("n²"), "{w}");
        assert!(index_background_warning(Measure::Alpha, 2.6, &n, &k).is_none());
    }

    #[test]
    fn alpha_without_eps_inf_says_the_index_is_not_fixed() {
        let (report, _) = recover(Measure::Alpha, None, 0.0);
        assert!(
            report.warnings.iter().any(|w| w.contains("no fijan el índice") || w.contains("ese 1 no sale de la medida")),
            "{:?}",
            report.warnings
        );
    }

    #[test]
    fn n_alone_fits_eps_inf() {
        let (report, _) = recover(Measure::N, None, 0.0);
        let got = report.eps_inf.unwrap();
        assert!((got - 2.5).abs() < 0.3, "ε∞ {got}");
        assert_eq!(report.eps_inf_note, "ajustado junto con ε₂");
    }

    #[test]
    fn eps2_peaks_land_on_the_oscillators() {
        let (report, _) = recover(Measure::K, Some(2.5), 0.0);
        assert!(report.peaks.iter().any(|p| (p.position - 300.0).abs() < 12.0), "{:?}", report.peaks);
        assert!(report.peaks.iter().any(|p| (p.position - 600.0).abs() < 25.0), "{:?}", report.peaks);
        assert_eq!(report.absorption.len(), report.omega.len());
        assert_eq!(report.sigma1.len(), report.omega.len());
        assert_eq!(report.sigma2.len(), report.omega.len());
        assert!(report.transmittance.is_empty());
        let i = report.omega.len() / 2;
        let v = Unit::Cm.wavenumber(report.omega[i]).unwrap();
        let expect = v * (report.eps_inf.unwrap() - report.eps1[i]) / 60.0;
        assert!((report.sigma2[i] - expect).abs() < 1e-8, "{} vs {expect}", report.sigma2[i]);
        let (film, _) = recover(Measure::K, Some(2.5), 1.0);
        assert_eq!(film.transmittance.len(), film.omega.len());
        assert_eq!(film.absorbance.len(), film.omega.len());
        assert!(film.transmittance.iter().all(|t| (0.0..=1.0).contains(t)));
    }

    #[test]
    fn alpha_without_units_is_refused() {
        let err = fit_text("1 10\n2 20\n3 15\n", &TextFit { measure: "alpha".into(), iter_max: 10, ..TextFit::default() }).unwrap_err();
        assert!(err.contains("cm⁻¹"), "{err}");
    }

    #[test]
    fn luminescence_in_nm_reports_peaks_in_ev() {
        let mut text = String::new();
        for i in 0..400 {
            let lambda = 700.0 + i as f64 * 0.5;
            let e = HC_EV_NM_TEST / lambda;
            let i_e = (-(e - 1.6) * (e - 1.6) / (2.0 * 0.02 * 0.02)).exp();
            let i_l = i_e * HC_EV_NM_TEST / (lambda * lambda);
            text.push_str(&format!("{lambda} {i_l}\n"));
        }
        let report = fit_text(
            &text,
            &TextFit { measure: "lum".into(), unit: "nm".into(), temperature: "300".into(), iter_max: 10, ..TextFit::default() },
        )
        .unwrap();
        assert_eq!(report.peaks.len(), 1, "{:?}", report.peaks);
        let p = &report.peaks[0];
        assert!((p.position - 1.6).abs() < 0.003, "pico {}", p.position);
        let fwhm = 2.0 * (2.0 * 2f64.ln()).sqrt() * 0.02;
        assert!((p.fwhm.unwrap() - fwhm).abs() < 0.003, "fwhm {:?}", p.fwhm);
        assert!((report.centroid.unwrap() - 1.6).abs() < 0.003);
        assert!(!report.vrs_x.is_empty());
        assert!(report.omega.is_empty());
    }

    const HC_EV_NM_TEST: f64 = crate::measure::HC_EV_NM;

    #[test]
    fn band_solver_matches_the_dense_one() {
        let mut b = Band::new(6);
        let mut d = Dense::new(6);
        let rows = [
            [1.0, 0.3, 0.1, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.4, 0.2, 0.0, 0.0],
            [0.0, 0.0, 1.5, 0.3, 0.1, 0.0],
            [0.0, 0.0, 0.0, 1.2, 0.5, 0.2],
            [0.0, 0.0, 0.0, 0.0, 1.8, 0.3],
            [0.0, 0.0, 0.0, 0.0, 0.0, 1.1],
        ];
        let as_row = |r: &[f64]| {
            let mut row = Row::default();
            for (j, v) in r.iter().enumerate() {
                if *v != 0.0 {
                    row.push(j, *v);
                }
            }
            row
        };
        for r in rows {
            b.add_row(&as_row(&r), 1.0);
            d.add_row(&as_row(&r), 1.0);
        }
        let rhs = [1.0, -2.0, 0.5, 3.0, 0.1, 2.0];
        let xb = b.solve_damped(0.2, &rhs).unwrap();
        let xd = d.solve_damped(0.2, &rhs).unwrap();
        for (p, q) in xb.iter().zip(&xd) {
            assert!((p - q).abs() < 1e-9, "{p} {q}");
        }
        let cb = b.covariance().unwrap();
        let cd = d.covariance().unwrap();
        for j in 0..6 {
            assert!((cb.diag(j) - cd.diag(j)).abs() < 1e-9);
        }
        let row = as_row(&[0.0, 0.0, 0.4, 0.6, 0.0, 0.0]);
        assert!((cb.quad(&row) - cd.quad(&row)).abs() < 1e-9);
    }

    #[test]
    fn ten_thousand_points_keep_one_anchor_each() {
        let x: Vec<f64> = (0..10_000).map(|i| 100.0 + i as f64 * 0.25).collect();
        let y: Vec<f64> = x.iter().map(|&w| lorentz(w).1).collect();
        let report = run_fit(&FitRequest {
            x,
            y,
            iter_max: 20,
            log_mesh: false,
            drude: None,
            smoothness: 0.08,
            negativity: 100.0,
            extended: false,
            sigma: vec![0.01; 10_000],
            errors_given: true,
            physics: Physics { unit: Unit::Cm, ..Physics::default() },
        })
        .unwrap();
        assert_eq!(report.points_used, 10_000);
        assert_eq!(report.mfit, 10_000);
        assert!(report.converged);
        assert!(report.chisq_dof < 1.0 || report.chisq < 10_000.0, "χ² {}", report.chisq);
        assert_eq!(report.eps2_err.len(), 10_000);
        assert!(report.eps2_err.iter().all(|e| *e > 0.0 && *e < 0.02));
    }

    #[test]
    fn dense_fit_rejects_excessive_work_before_allocating_matrices() {
        let points = 300;
        let result = run_fit(&FitRequest {
            x: (1..=points).map(|value| value as f64).collect(),
            y: vec![1.0; points],
            iter_max: 1000,
            log_mesh: false,
            drude: None,
            smoothness: 0.08,
            negativity: 100.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics { measure: Measure::N, ..Physics::default() },
        });
        assert!(result.unwrap_err().contains("demasiados recursos"));
    }

    #[test]
    fn fit_rejects_more_than_maximum_points() {
        let points = MAX_FIT_POINTS + 1;
        let result = run_fit(&FitRequest {
            x: (1..=points).map(|value| value as f64).collect(),
            y: vec![1.0; points],
            iter_max: 1,
            log_mesh: false,
            drude: None,
            smoothness: 0.08,
            negativity: 100.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics::default(),
        });
        assert!(result.unwrap_err().contains("hasta 10000 puntos"));
    }

    #[test]
    fn aux_g_matches_identity() {
        let (x, y) = (1.2, 3.4);
        let s = kuzmenko_g(x, y) + kuzmenko_g(y, x);
        let two = 2.0 * (x + y) * (x + y).abs().ln();
        assert!((s - two).abs() < 1e-9);
        assert_eq!(kuzmenko_g(0.0, 0.0), 0.0);
    }

    #[test]
    fn demo_spectrum_is_finite() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y = vec![0.2, 0.35, 0.5, 0.62, 0.55, 0.48, 0.4, 0.3];
        let report = run_fit(&FitRequest {
            x,
            y,
            iter_max: 12,
            log_mesh: false,
            drude: None,
            smoothness: 0.08,
            negativity: 100.0,
            extended: true,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics::default(),
        }).unwrap();
        assert!(report.chisq.is_finite());
        assert_eq!(report.eps1.len(), report.eps2.len());
        assert!(report.eps2.iter().all(|v| v.is_finite()));
        assert!(!report.ext_omega.is_empty());
        assert!(report.points_used >= 3);
    }

    #[test]
    fn skips_text_headers_and_merges_duplicate_omega() {
        let text = "# comentario\nomega epsilon\n1.0 0.2\n1.0 0.4\n2.0 0.5\n3.0 0.3\n";
        let (x, y, _, _) = parse_spectrum(text).unwrap();
        assert_eq!(x.len(), 4);
        let report = run_fit(&FitRequest {
            x,
            y,
            iter_max: 6,
            log_mesh: false,
            drude: None,
            smoothness: 0.0,
            negativity: 0.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics::default(),
        })
        .unwrap();
        assert!(report.points_used <= 3);
        assert!(report.chisq.is_finite());
    }

    #[test]
    fn rejects_non_positive_frequency() {
        let err = run_fit(&FitRequest {
            x: vec![1.0, 0.0, 3.0],
            y: vec![0.1, 0.2, 0.3],
            iter_max: 4,
            log_mesh: false,
            drude: None,
            smoothness: 0.0,
            negativity: 0.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics::default(),
        }).unwrap_err();
        assert!(err.contains("positiva"));
    }

    #[test]
    fn kk_kernel_matches_principal_value() {
        // Triángulo de altura 1 entre 2, 3 y 5. Fuera del soporte, en ω = 8,
        // (2/π) P∫ ω'ε₂/(ω'²−ω²) dω' = −0.0620516962.
        let anchors = [2.0, 3.0, 5.0];
        let a = [0.0, 1.0, 0.0];
        let eps1 = dkkt_eps1(8.0, &anchors, &a);
        assert!((eps1 - 1.0 + 0.0620516962).abs() < 1e-8);
    }

    #[test]
    fn optics_rebuilds_epsilon() {
        let (n, k, r) = optics(2.0, -0.4);
        assert!((n * n - k * k - 2.0).abs() < 1e-9);
        assert!((2.0 * n * k + 0.4).abs() < 1e-9);
        assert!((0.0..=1.0).contains(&r));
    }

    #[test]
    fn measured_edges_keep_their_epsilon() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y = vec![0.2, 0.35, 0.5, 0.62, 0.55, 0.48, 0.4, 0.3];
        let report = run_fit(&FitRequest {
            x: x.clone(),
            y: y.clone(),
            iter_max: 4,
            log_mesh: false,
            drude: None,
            smoothness: 0.0,
            negativity: 0.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics::default(),
        })
        .unwrap();
        assert!(report.warnings.iter().any(|w| w.contains("ε₂ no queda fijada por los datos")));
        assert!(report.chisq < 1e-8, "χ² de datos {}", report.chisq);
        assert!((report.eps2[0] - y[0]).abs() < 1e-6);
        assert!((report.eps2.last().copied().unwrap() - y[7]).abs() < 1e-6);
        for i in 0..report.eps1.len() {
            let (n, k, _) = optics(report.eps1[i], report.eps2[i]);
            assert!((n - report.n_index[i]).abs() < 1e-9);
            assert!((2.0 * n * k - report.eps2[i]).abs() < 1e-8);
        }
    }

    #[test]
    fn drude_width_follows_the_spectrum() {
        let d = suggest_drude(&[0.5, 1.0, 2.0, 4.0], &[1.0, 0.5, 0.2, 0.1]);
        assert!((d.gamma - 0.25).abs() < 1e-9, "γ {}", d.gamma);
        assert!(eps2_drude(1.0, &d) > 0.0);
        assert!(eps1_drude(1.0, &d) < d.eps_inf);
    }

    #[test]
    fn smoothness_leaves_the_measured_edges() {
        let y = vec![0.2, 0.35, 0.5, 0.62, 0.55, 0.48, 0.4, 0.3];
        let report = run_fit(&FitRequest {
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            y: y.clone(),
            iter_max: 8,
            log_mesh: false,
            drude: None,
            smoothness: 0.08,
            negativity: 0.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics::default(),
        })
        .unwrap();
        assert!((report.eps2[0] - y[0]).abs() < 0.015, "inicio {}", report.eps2[0]);
        assert!((report.eps2.last().copied().unwrap() - y[7]).abs() < 0.015, "fin {}", report.eps2.last().unwrap());
    }

    #[test]
    fn log_mesh_fills_the_wide_end() {
        let x: Vec<f64> = (0..40).map(|i| 10f64.powf(-2.0 + i as f64 * 0.15)).collect();
        let y = vec![0.4; x.len()];
        let report = run_fit(&FitRequest {
            x,
            y,
            iter_max: 2,
            log_mesh: true,
            drude: None,
            smoothness: 0.0,
            negativity: 0.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics::default(),
        })
        .unwrap();
        assert_eq!(report.points_used, 40);
        assert!(report.n_anchor >= 40, "anclas {}", report.n_anchor);
        assert!(
            report.model_eps2.iter().all(|v| (v - 0.4).abs() < 1e-3),
            "modelo {:?}",
            report.model_eps2
        );
    }

    #[test]
    fn a_narrow_peak_stays_without_using_the_step_cap() {
        let mut x = Vec::new();
        let mut y = Vec::new();
        for i in 0..40 {
            x.push(10f64.powf(-1.0 + i as f64 * 0.1));
            y.push(if i == 15 { 1.0 } else { 0.05 });
        }
        let report = run_fit(&FitRequest {
            x,
            y,
            iter_max: 8,
            log_mesh: false,
            drude: None,
            smoothness: 0.0,
            negativity: 0.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics::default(),
        })
        .unwrap();
        let peak = report.model_eps2.iter().copied().fold(0.0_f64, f64::max);
        assert!(peak > 0.95, "pico {peak}");
        assert!(report.iters_used < 20, "pasos {}", report.iters_used);
        assert!(report.converged);
    }

    #[test]
    fn a_negative_dip_needs_the_penalty_at_zero() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y = vec![0.2, 0.3, 0.4, -0.5, 0.4, 0.3, 0.25, 0.2];
        let free = run_fit(&FitRequest {
            x: x.clone(),
            y: y.clone(),
            iter_max: 12,
            log_mesh: false,
            drude: None,
            smoothness: 0.0,
            negativity: 0.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics::default(),
        })
        .unwrap();
        let held = run_fit(&FitRequest {
            x,
            y,
            iter_max: 12,
            log_mesh: false,
            drude: None,
            smoothness: 0.0,
            negativity: 100.0,
            extended: false,
            sigma: Vec::new(),
            errors_given: false,
            physics: Physics::default(),
        })
        .unwrap();
        assert!(free.model_eps2[3] < -0.4, "libre {}", free.model_eps2[3]);
        assert!(held.model_eps2[3] > -0.05, "castigado {}", held.model_eps2[3]);
    }

    #[test]
    fn experimental_error_is_the_confidence_bar() {
        let text = "1 0.20 0.02\n2 0.35 0.02\n3 0.50 0.02\n4 0.62 0.02\n5 0.55 0.02\n6 0.48 0.02\n7 0.40 0.02\n8 0.30 0.02\n";
        let (x, y, sigma, errors_given) = parse_spectrum(text).unwrap();
        assert!(errors_given);
        let report = run_fit(&FitRequest {
            x,
            y,
            iter_max: 8,
            log_mesh: false,
            drude: None,
            smoothness: 0.0,
            negativity: 0.0,
            extended: false,
            sigma,
            errors_given,
            physics: Physics::default(),
        })
        .unwrap();
        assert!(report.chisq < 1e-6, "χ² {}", report.chisq);
        assert_eq!(report.eps2_err.len(), report.data_x.len());
        for s in &report.eps2_err {
            assert!((s - 0.02).abs() < 1e-6, "barra {s}");
        }
        assert_eq!(report.coef_err.len(), report.omega.len());
    }

    #[test]
    fn a_loose_point_yields_to_smoothness() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![0.2, 0.2, 5.0, 0.2, 0.2];
        let loose = run_fit(&FitRequest {
            x: x.clone(),
            y: y.clone(),
            iter_max: 16,
            log_mesh: false,
            drude: None,
            smoothness: 0.08,
            negativity: 0.0,
            extended: false,
            sigma: vec![0.01, 0.01, 10.0, 0.01, 0.01],
            errors_given: true,
            physics: Physics::default(),
        })
        .unwrap();
        let tight = run_fit(&FitRequest {
            x,
            y,
            iter_max: 16,
            log_mesh: false,
            drude: None,
            smoothness: 0.08,
            negativity: 0.0,
            extended: false,
            sigma: vec![0.01; 5],
            errors_given: true,
            physics: Physics::default(),
        })
        .unwrap();
        assert!(loose.model_eps2[2] < 1.0, "error grande {}", loose.model_eps2[2]);
        assert!(tight.model_eps2[2] > 4.0, "error chico {}", tight.model_eps2[2]);
    }
}
