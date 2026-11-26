//! Motor denso del análisis variacional Kramers–Kronig restringido (Kuzmenko, 2005).
//!
//! Para R, T, n, k, α y ε₁ la medida en cada frecuencia depende de todas las alturas
//! de la malla triangular a través de ε₁ = ε∞ + K·A. Este módulo resuelve ese problema
//! sin recortar la malla a unas pocas anclas:
//!
//! - el núcleo K (ec. 13 del artículo) se calcula al vuelo, fila a fila, así que la
//!   memoria crece con M² y no con N·M;
//! - las ecuaciones normales JᵀJ se acumulan por bloques de filas y en paralelo;
//! - el paso de Levenberg–Marquardt se resuelve con Gauss–Jordan o con Cholesky;
//! - la malla se recorre de gruesa a fina. Cada malla contiene a la anterior, de modo
//!   que la función ε₂ pasa intacta de un nivel al siguiente y la malla completa
//!   arranca ya cerca del mínimo.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::fit::FitProgressCurve;
use crate::measure::{quantity_grad, Measure};

const PI: f64 = std::f64::consts::PI;

/// Cómo se reparten las anclas entre los datos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshMode {
    /// Completa si cabe en el presupuesto de cómputo; si no, adaptativa.
    #[default]
    Auto,
    /// Un ancla por cada intervalo entre datos.
    Full,
    /// Un ancla cada dos intervalos: N ≈ N_exp/2, el óptimo que cita Kuzmenko.
    Half,
    /// Parte de una malla gruesa y la afina solo donde el residuo tiene estructura.
    Adaptive,
}

impl MeshMode {
    pub const ALL: [MeshMode; 4] = [MeshMode::Auto, MeshMode::Full, MeshMode::Half, MeshMode::Adaptive];

    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim();
        Self::ALL.into_iter().find(|m| m.key() == raw).unwrap_or_default()
    }

    pub fn key(self) -> &'static str {
        match self {
            MeshMode::Auto => "auto",
            MeshMode::Full => "full",
            MeshMode::Half => "half",
            MeshMode::Adaptive => "adaptive",
        }
    }
}

/// Cómo se resuelve el sistema lineal de cada paso de Levenberg–Marquardt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Solver {
    /// Gauss–Jordan en mallas pequeñas y Cholesky en las grandes.
    #[default]
    Auto,
    /// Eliminación de Gauss–Jordan con pivote (Numerical Recipes §2.1).
    GaussJordan,
    /// Factorización de Cholesky por bloques.
    Cholesky,
}

impl Solver {
    pub const ALL: [Solver; 3] = [Solver::Auto, Solver::GaussJordan, Solver::Cholesky];

    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim();
        Self::ALL.into_iter().find(|m| m.key() == raw).unwrap_or_default()
    }

    pub fn key(self) -> &'static str {
        match self {
            Solver::Auto => "auto",
            Solver::GaussJordan => "gauss_jordan",
            Solver::Cholesky => "cholesky",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Solver::Auto => "auto",
            Solver::GaussJordan => "Gauss–Jordan",
            Solver::Cholesky => "Cholesky",
        }
    }
}

/// Qué hace ε₂ fuera del rango medido.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Extrapolation {
    /// Sigue con colas suaves: σ₁ constante hacia ω → 0 y caída ω⁻³ por encima del rango.
    #[default]
    Tails,
    /// Baja en línea recta hasta cero en un nodo de borde, como la malla del trabajo de grado.
    Zero,
}

impl Extrapolation {
    pub const ALL: [Extrapolation; 2] = [Extrapolation::Tails, Extrapolation::Zero];

    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim();
        Self::ALL.into_iter().find(|m| m.key() == raw).unwrap_or_default()
    }

    pub fn key(self) -> &'static str {
        match self {
            Extrapolation::Tails => "tails",
            Extrapolation::Zero => "zero",
        }
    }
}

/// Tope de anclas interiores. La matriz del paso ocupa 8·M² bytes dos veces.
pub const MAX_ANCHORS: usize = 6_000;
/// Presupuesto N·M² de una acumulación de JᵀJ.
#[cfg(not(target_arch = "wasm32"))]
const WORK_BUDGET: f64 = 2.4e11;
/// En el navegador el motor corre en un solo hilo y sin AVX: el presupuesto es veinte veces menor.
#[cfg(target_arch = "wasm32")]
const WORK_BUDGET: f64 = 1.2e10;
/// Con `Solver::Auto`, hasta aquí se usa Gauss–Jordan.
const AUTO_GAUSS_JORDAN_MAX: usize = 2_500;
/// Anclas del nivel más grueso de la continuación.
const COARSE_ANCHORS: usize = 192;
/// Pasos de Levenberg–Marquardt en cada nivel intermedio.
const COARSE_STEPS: usize = 4;
/// Media ventana, en puntos, de la estimación local del ruido.
const NOISE_WINDOW: usize = 24;

/// Anclas interiores que caben en el presupuesto para `points` datos.
pub fn anchor_cap(points: usize) -> usize {
    let by_work = (WORK_BUDGET / points.max(1) as f64).sqrt();
    (by_work as usize).clamp(64, MAX_ANCHORS)
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    /// Tope de hilos de los ajustes que corren en este hilo. 0 = sin tope (todos los núcleos).
    static THREAD_BUDGET: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Núcleos que ve el proceso, acotados a 16. En WASM, uno.
pub fn cores() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        1
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).clamp(1, 16)
    }
}

/// Ejecuta `f` con un tope de `threads` hilos para los cálculos paralelos de este hilo.
///
/// Sirve al servidor cuando varios ajustes corren a la vez: cada uno recibe una parte de los
/// núcleos en lugar de lanzar todos los hilos y pelearse por ellos. Los hilos que `f` lance no
/// heredan el tope, pero el motor decide cuántos lanzar en el hilo que lo llama.
pub fn with_thread_budget<R>(threads: usize, f: impl FnOnce() -> R) -> R {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = threads;
        f()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        struct Restore(usize);
        impl Drop for Restore {
            fn drop(&mut self) {
                THREAD_BUDGET.with(|budget| budget.set(self.0));
            }
        }
        let _restore = Restore(THREAD_BUDGET.with(|budget| budget.replace(threads.max(1))));
        f()
    }
}

/// Hilos que merece un trabajo de `work` operaciones. En WASM siempre uno.
pub(crate) fn worker_count(work: f64) -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = work;
        1
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if work < 4.0e6 {
            1
        } else {
            match THREAD_BUDGET.with(std::cell::Cell::get) {
                0 => cores(),
                budget => budget.min(cores()),
            }
        }
    }
}

#[inline(always)]
fn fma<const FMA: bool>(a: f64, b: f64, c: f64) -> f64 {
    if FMA {
        a.mul_add(b, c)
    } else {
        a * b + c
    }
}

#[inline(always)]
fn dot_impl<const FMA: bool>(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    let (a, b) = (&a[..n], &b[..n]);
    let mut acc = [0.0f64; 8];
    let mut ca = a.chunks_exact(8);
    let mut cb = b.chunks_exact(8);
    for (x, y) in (&mut ca).zip(&mut cb) {
        for l in 0..8 {
            acc[l] = fma::<FMA>(x[l], y[l], acc[l]);
        }
    }
    let mut s = (acc[0] + acc[4]) + (acc[1] + acc[5]) + (acc[2] + acc[6]) + (acc[3] + acc[7]);
    for (x, y) in ca.remainder().iter().zip(cb.remainder()) {
        s += x * y;
    }
    s
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    dot_impl::<false>(a, b)
}

/// y ← y − a·x
#[inline(always)]
fn axpy_neg_impl<const FMA: bool>(y: &mut [f64], a: f64, x: &[f64]) {
    for (yk, xk) in y.iter_mut().zip(x) {
        *yk = fma::<FMA>(-a, *xk, *yk);
    }
}

/// Suma JbᵀJb en las filas j0.. de H (triángulo inferior). `hs` son esas filas, de ancho m.
#[inline(always)]
fn syrk_impl<const FMA: bool>(hs: &mut [f64], m: usize, j0: usize, jb: &[f64], nb: usize) {
    for (dj, hrow) in hs.chunks_exact_mut(m).enumerate() {
        let j = j0 + dj;
        let len = j + 1;
        let hrow = &mut hrow[..len];
        let mut i = 0;
        while i + 4 <= nb {
            let r0 = &jb[i * m..i * m + len];
            let r1 = &jb[(i + 1) * m..(i + 1) * m + len];
            let r2 = &jb[(i + 2) * m..(i + 2) * m + len];
            let r3 = &jb[(i + 3) * m..(i + 3) * m + len];
            let (a0, a1, a2, a3) = (r0[j], r1[j], r2[j], r3[j]);
            for ((((h, x0), x1), x2), x3) in hrow.iter_mut().zip(r0).zip(r1).zip(r2).zip(r3) {
                *h = fma::<FMA>(a0, *x0, fma::<FMA>(a1, *x1, fma::<FMA>(a2, *x2, fma::<FMA>(a3, *x3, *h))));
            }
            i += 4;
        }
        while i < nb {
            let r0 = &jb[i * m..i * m + len];
            let a0 = r0[j];
            if a0 != 0.0 {
                for (h, x0) in hrow.iter_mut().zip(r0) {
                    *h = fma::<FMA>(a0, *x0, *h);
                }
            }
            i += 1;
        }
    }
}

/// Actualización de la parte restante de Cholesky: fila i ← fila i − P[i]·P[j]ᵀ.
#[inline(always)]
fn trailing_impl<const FMA: bool>(rows: &mut [f64], n: usize, i0: usize, p1: usize, panel: &[f64], nbw: usize) {
    for (di, row) in rows.chunks_exact_mut(n).enumerate() {
        let i = i0 + di;
        let pi = &panel[(i - p1) * nbw..(i - p1 + 1) * nbw];
        for (dj, v) in row[p1..=i].iter_mut().enumerate() {
            let pj = &panel[dj * nbw..(dj + 1) * nbw];
            *v -= dot_impl::<FMA>(pi, pj);
        }
    }
}

/// Paso de eliminación de Gauss–Jordan sobre un grupo de filas.
#[inline(always)]
fn eliminate_impl<const FMA: bool>(rows: &mut [f64], n: usize, first: usize, piv: usize, prow: &[f64], rhs: &mut [f64], bpiv: f64) {
    for (dl, row) in rows.chunks_exact_mut(n).enumerate() {
        let ll = first + dl;
        if ll == piv {
            continue;
        }
        let dum = row[piv];
        if dum == 0.0 {
            continue;
        }
        row[piv] = 0.0;
        axpy_neg_impl::<FMA>(row, dum, prow);
        rhs[dl] -= bpiv * dum;
    }
}

/// Gauss–Jordan por bloques: a cada fila se le restan de una vez las `k1 − k0` filas pivote
/// ya reducidas. `upper` guarda esas filas desde la columna `k1`, contiguas y de ancho `rest`.
#[inline(always)]
fn block_eliminate_impl<const FMA: bool>(rows: &mut [f64], n: usize, k0: usize, k1: usize, upper: &[f64], rest: usize) {
    let nb = k1 - k0;
    for row in rows.chunks_exact_mut(n) {
        let (head, tail) = row.split_at_mut(k1);
        let mult = &head[k0..k1];
        let mut p = 0;
        while p + 4 <= nb {
            let (m0, m1, m2, m3) = (-mult[p], -mult[p + 1], -mult[p + 2], -mult[p + 3]);
            let u0 = &upper[p * rest..(p + 1) * rest];
            let u1 = &upper[(p + 1) * rest..(p + 2) * rest];
            let u2 = &upper[(p + 2) * rest..(p + 3) * rest];
            let u3 = &upper[(p + 3) * rest..(p + 4) * rest];
            for ((((t, x0), x1), x2), x3) in tail.iter_mut().zip(u0).zip(u1).zip(u2).zip(u3) {
                *t = fma::<FMA>(m0, *x0, fma::<FMA>(m1, *x1, fma::<FMA>(m2, *x2, fma::<FMA>(m3, *x3, *t))));
            }
            p += 4;
        }
        while p < nb {
            let m0 = mult[p];
            if m0 != 0.0 {
                axpy_neg_impl::<FMA>(tail, m0, &upper[p * rest..(p + 1) * rest]);
            }
            p += 1;
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod simd {
    #[target_feature(enable = "avx2,fma")]
    pub unsafe fn block_eliminate(rows: &mut [f64], n: usize, k0: usize, k1: usize, upper: &[f64], rest: usize) {
        super::block_eliminate_impl::<true>(rows, n, k0, k1, upper, rest)
    }

    #[target_feature(enable = "avx2,fma")]
    pub unsafe fn syrk(hs: &mut [f64], m: usize, j0: usize, jb: &[f64], nb: usize) {
        super::syrk_impl::<true>(hs, m, j0, jb, nb)
    }

    #[target_feature(enable = "avx2,fma")]
    pub unsafe fn trailing(rows: &mut [f64], n: usize, i0: usize, p1: usize, panel: &[f64], nbw: usize) {
        super::trailing_impl::<true>(rows, n, i0, p1, panel, nbw)
    }

    #[target_feature(enable = "avx2,fma")]
    pub unsafe fn eliminate(rows: &mut [f64], n: usize, first: usize, piv: usize, prow: &[f64], rhs: &mut [f64], bpiv: f64) {
        super::eliminate_impl::<true>(rows, n, first, piv, prow, rhs, bpiv)
    }

    pub fn available() -> bool {
        std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("fma")
    }
}

fn syrk(hs: &mut [f64], m: usize, j0: usize, jb: &[f64], nb: usize) {
    #[cfg(target_arch = "x86_64")]
    if simd::available() {
        // SAFETY: la CPU anuncia AVX2 y FMA.
        return unsafe { simd::syrk(hs, m, j0, jb, nb) };
    }
    syrk_impl::<false>(hs, m, j0, jb, nb)
}

fn trailing(rows: &mut [f64], n: usize, i0: usize, p1: usize, panel: &[f64], nbw: usize) {
    #[cfg(target_arch = "x86_64")]
    if simd::available() {
        // SAFETY: la CPU anuncia AVX2 y FMA.
        return unsafe { simd::trailing(rows, n, i0, p1, panel, nbw) };
    }
    trailing_impl::<false>(rows, n, i0, p1, panel, nbw)
}

fn eliminate(rows: &mut [f64], n: usize, first: usize, piv: usize, prow: &[f64], rhs: &mut [f64], bpiv: f64) {
    #[cfg(target_arch = "x86_64")]
    if simd::available() {
        // SAFETY: la CPU anuncia AVX2 y FMA.
        return unsafe { simd::eliminate(rows, n, first, piv, prow, rhs, bpiv) };
    }
    eliminate_impl::<false>(rows, n, first, piv, prow, rhs, bpiv)
}

fn block_eliminate(rows: &mut [f64], n: usize, k0: usize, k1: usize, upper: &[f64], rest: usize) {
    #[cfg(target_arch = "x86_64")]
    if simd::available() {
        // SAFETY: la CPU anuncia AVX2 y FMA.
        return unsafe { simd::block_eliminate(rows, n, k0, k1, upper, rest) };
    }
    block_eliminate_impl::<false>(rows, n, k0, k1, upper, rest)
}

/// Reparte las filas [lo, hi) de una matriz triangular en `parts` tramos de trabajo parecido.
fn triangular_cuts(lo: usize, hi: usize, parts: usize) -> Vec<usize> {
    let mut cuts = Vec::with_capacity(parts + 1);
    cuts.push(lo);
    let (a, b) = (lo as f64, hi as f64);
    for t in 1..parts {
        let frac = t as f64 / parts as f64;
        let x = (a * a + frac * (b * b - a * a)).sqrt().round() as usize;
        cuts.push(x.clamp(*cuts.last().unwrap(), hi));
    }
    cuts.push(hi);
    cuts
}

/// Ejecuta `job(inicio, filas)` sobre tramos disjuntos de `data` (filas de ancho `width`).
fn par_rows<F>(data: &mut [f64], width: usize, cuts: &[usize], job: F)
where
    F: Fn(usize, &mut [f64]) + Sync,
{
    if cuts.len() <= 2 {
        job(cuts[0], data);
        return;
    }
    std::thread::scope(|scope| {
        let mut rest = data;
        for pair in cuts.windows(2) {
            let (start, end) = (pair[0], pair[1]);
            let (mine, tail) = rest.split_at_mut((end - start) * width);
            rest = tail;
            if end > start {
                let job = &job;
                scope.spawn(move || job(start, mine));
            }
        }
    });
}

/// Factoriza A = L·Lᵀ (triángulo inferior, almacenamiento cuadrado por filas) y resuelve A·x = b.
/// Devuelve `false` si la matriz no es definida positiva.
pub(crate) fn cholesky_solve(a: &mut [f64], n: usize, b: &mut [f64], threads: usize) -> bool {
    const NB: usize = 64;
    let mut panel: Vec<f64> = Vec::new();
    let mut p0 = 0;
    while p0 < n {
        let p1 = (p0 + NB).min(n);
        let nbw = p1 - p0;
        // Bloque diagonal.
        for i in p0..p1 {
            for j in p0..=i {
                let (head, tail) = a.split_at_mut(i * n);
                let ri = &mut tail[..n];
                let s = if j == i {
                    ri[j] - dot(&ri[p0..j], &ri[p0..j])
                } else {
                    let rj = &head[j * n..j * n + n];
                    ri[j] - dot(&ri[p0..j], &rj[p0..j])
                };
                if j == i {
                    if !(s > 0.0) || !s.is_finite() {
                        return false;
                    }
                    ri[j] = s.sqrt();
                } else {
                    ri[j] = s / head[j * n + j];
                }
            }
        }
        if p1 == n {
            break;
        }
        let (top, bottom) = a.split_at_mut(p1 * n);
        let below = n - p1;
        let workers = if (below * nbw * nbw) < 200_000 { 1 } else { threads };
        // Columnas del panel en las filas de abajo.
        let even: Vec<usize> = (0..=workers).map(|t| p1 + below * t / workers).collect();
        par_rows(bottom, n, &even, |_, rows| {
            for row in rows.chunks_exact_mut(n) {
                for j in p0..p1 {
                    let rj = &top[j * n..j * n + n];
                    let s = row[j] - dot(&row[p0..j], &rj[p0..j]);
                    row[j] = s / rj[j];
                }
            }
        });
        // Copia compacta del panel: las filas quedan contiguas para la actualización.
        panel.clear();
        panel.reserve(below * nbw);
        for row in bottom.chunks_exact(n) {
            panel.extend_from_slice(&row[p0..p1]);
        }
        let cuts = triangular_cuts(p1, n, workers);
        let panel_ref = &panel;
        par_rows(bottom, n, &cuts, |i0, rows| trailing(rows, n, i0, p1, panel_ref, nbw));
        p0 = p1;
    }
    for i in 0..n {
        let row = &a[i * n..i * n + i + 1];
        b[i] = (b[i] - dot(&row[..i], &b[..i])) / row[i];
    }
    for i in (0..n).rev() {
        let row = &a[i * n..i * n + i + 1];
        let x = b[i] / row[i];
        b[i] = x;
        for (bk, lk) in b[..i].iter_mut().zip(&row[..i]) {
            *bk -= lk * x;
        }
    }
    b.iter().all(|v| v.is_finite())
}

/// Gauss–Jordan sobre una matriz simétrica definida positiva (almacenamiento cuadrado completo).
/// Deja en `a` la inversa y en `b` la solución. En una matriz así el elemento mayor de la parte
/// aún no reducida está en la diagonal, de modo que el pivote total de Numerical Recipes se
/// busca solo ahí y no hacen falta intercambios de filas.
pub(crate) fn gauss_jordan(a: &mut [f64], n: usize, b: &mut [f64], threads: usize) -> bool {
    let mut done = vec![false; n];
    let mut prow = vec![0.0; n];
    let workers = if n < 400 { 1 } else { threads };
    let cuts: Vec<usize> = (0..=workers).map(|t| n * t / workers).collect();
    for _ in 0..n {
        let mut big = 0.0;
        let mut piv = usize::MAX;
        for j in 0..n {
            if !done[j] {
                let v = a[j * n + j].abs();
                if v >= big {
                    big = v;
                    piv = j;
                }
            }
        }
        if piv == usize::MAX || !(big > 1e-300) || !big.is_finite() {
            return false;
        }
        done[piv] = true;
        let pivinv = 1.0 / a[piv * n + piv];
        a[piv * n + piv] = 1.0;
        for v in a[piv * n..(piv + 1) * n].iter_mut() {
            *v *= pivinv;
        }
        b[piv] *= pivinv;
        prow.copy_from_slice(&a[piv * n..(piv + 1) * n]);
        let bpiv = b[piv];
        let prow_ref = &prow;
        if workers <= 1 {
            eliminate(a, n, 0, piv, prow_ref, b, bpiv);
        } else {
            std::thread::scope(|scope| {
                let mut rest_a = &mut a[..];
                let mut rest_b = &mut b[..];
                for pair in cuts.windows(2) {
                    let (start, end) = (pair[0], pair[1]);
                    let (mine_a, tail_a) = rest_a.split_at_mut((end - start) * n);
                    let (mine_b, tail_b) = rest_b.split_at_mut(end - start);
                    rest_a = tail_a;
                    rest_b = tail_b;
                    if end > start {
                        scope.spawn(move || eliminate(mine_a, n, start, piv, prow_ref, mine_b, bpiv));
                    }
                }
            });
        }
    }
    b.iter().all(|v| v.is_finite())
}

/// Resuelve A·x = b por eliminación de Gauss–Jordan, sin formar la inversa: cada columna
/// pivote se anula en todas las demás filas, por encima y por debajo, y al terminar la matriz
/// es la identidad y `b` es la solución. Las filas pivote se tratan en bloques de `NB` para que
/// cada pasada por la matriz reste `NB` filas a la vez en lugar de una; el resultado es el
/// mismo, la memoria se recorre `NB` veces menos. Como en `gauss_jordan`, la matriz es
/// simétrica definida positiva y el pivote se toma de la diagonal.
pub(crate) fn gauss_jordan_solve(a: &mut [f64], n: usize, b: &mut [f64], threads: usize) -> bool {
    const NB: usize = 48;
    let mut upper: Vec<f64> = Vec::new();
    let mut k0 = 0;
    while k0 < n {
        let k1 = (k0 + NB).min(n);
        let nb = k1 - k0;
        let rest = n - k1;
        // Gauss–Jordan sin bloques dentro de las filas pivote, columnas k0..n.
        for p in k0..k1 {
            let pivot = a[p * n + p];
            if !(pivot.abs() > 1e-300) || !pivot.is_finite() {
                return false;
            }
            let inv = 1.0 / pivot;
            for v in a[p * n + p..(p + 1) * n].iter_mut() {
                *v *= inv;
            }
            b[p] *= inv;
            let bp = b[p];
            let (before, from) = a.split_at_mut(p * n);
            let (prow, after) = from.split_at_mut(n);
            let prow = &prow[p..];
            for (i, row) in before[k0 * n..].chunks_exact_mut(n).enumerate() {
                let f = row[p];
                if f != 0.0 {
                    axpy_neg_impl::<false>(&mut row[p..], f, prow);
                    b[k0 + i] -= f * bp;
                }
            }
            for (i, row) in after[..(k1 - p - 1) * n].chunks_exact_mut(n).enumerate() {
                let f = row[p];
                if f != 0.0 {
                    axpy_neg_impl::<false>(&mut row[p..], f, prow);
                    b[p + 1 + i] -= f * bp;
                }
            }
        }
        // Lado derecho de las demás filas.
        for i in (0..k0).chain(k1..n) {
            let mult = &a[i * n + k0..i * n + k1];
            b[i] -= dot(mult, &b[k0..k1]);
        }
        if rest > 0 {
            upper.clear();
            upper.reserve(nb * rest);
            for p in k0..k1 {
                upper.extend_from_slice(&a[p * n + k1..(p + 1) * n]);
            }
            let upper_ref = &upper;
            let (top, from) = a.split_at_mut(k0 * n);
            let (_, bottom) = from.split_at_mut(nb * n);
            let workers = if (n - nb) * nb * rest < 400_000 { 1 } else { threads.max(1) };
            for part in [top, bottom] {
                let count = part.len() / n;
                if count == 0 {
                    continue;
                }
                let share = (workers * count).div_ceil(n - nb).clamp(1, workers);
                let cuts: Vec<usize> = (0..=share).map(|t| count * t / share).collect();
                par_rows(part, n, &cuts, |_, rows| block_eliminate(rows, n, k0, k1, upper_ref, rest));
            }
        }
        k0 = k1;
    }
    b.iter().all(|v| v.is_finite())
}

/// El problema: qué se midió y con qué pesos.
#[derive(Clone, Copy)]
pub(crate) struct Problem<'a> {
    pub measure: Measure,
    pub w: &'a [f64],
    pub y: &'a [f64],
    pub inv_var: &'a [f64],
    pub nu: &'a [f64],
    pub d_cm: f64,
    pub e1_fixed: &'a [f64],
    pub e2_fixed: &'a [f64],
    pub fit_eps_inf: bool,
    pub eps_inf_bounds: Option<(f64, f64)>,
    /// El espectro trajo el error de cada punto.
    pub errors_given: bool,
    pub smoothness: f64,
    pub negativity: f64,
}

/// Anclas, pesos cerrados del núcleo KK y la base triangular de cada dato.
pub(crate) struct Mesh {
    pub anchors: Vec<f64>,
    cm: Vec<f64>,
    c0: Vec<f64>,
    cp: Vec<f64>,
    /// Para cada dato: parámetro de la izquierda, parámetro de la derecha y sus pesos.
    left: Vec<usize>,
    right: Vec<usize>,
    b_left: Vec<f64>,
    b_right: Vec<f64>,
    /// Ancla a la izquierda de cada dato.
    seg: Vec<usize>,
    /// ε₂ sigue fuera de la malla con las colas `tail_low` y `tail_high` en vez de caer a cero.
    pub tails: bool,
}

/// Cola baja: ε₂(ω) = A·a/ω para ω < a, la de una conductividad σ₁ constante.
#[inline]
fn tail_low_shape(om: f64, a: f64) -> f64 {
    if om > 0.0 {
        a / om
    } else {
        0.0
    }
}

/// Cola alta: ε₂(ω) = A·(a/ω)³ para ω > a, la caída de portadores libres lejos de su ancho.
#[inline]
fn tail_high_shape(om: f64, a: f64) -> f64 {
    let u = a / om;
    u * u * u
}

/// Parte de ε₁ que da la cola baja de altura 1, junto con el término de borde de la malla
/// lineal que termina en `a`: (1/π)[(a/ω − 1) ln|a − ω| − (a/ω + 1) ln(a + ω) − 2].
#[inline]
fn tail_low_kernel(om: f64, a: f64) -> f64 {
    let u = a / om;
    let near = (a - om).abs();
    let first = if near > 0.0 { (u - 1.0) * near.ln() } else { 0.0 };
    (first - (u + 1.0) * (a + om).ln() - 2.0) / PI
}

/// Lo mismo para la cola alta:
/// (1/π)[(1 − a³/ω³) ln|a − ω| + (1 + a³/ω³) ln(a + ω) − 2a²/ω² + 2].
#[inline]
fn tail_high_kernel(om: f64, a: f64) -> f64 {
    let u = om / a;
    if u < 0.05 {
        // Serie en u = ω/a: los términos 2/u² se cancelan y la resta directa pierde cifras.
        let u2 = u * u;
        let series = 2.0 / 3.0 + u2 * (2.0 / 5.0 + u2 * (2.0 / 7.0 + u2 * (2.0 / 9.0)));
        let logs = (a - om).ln() + (a + om).ln();
        return (logs + series + 2.0) / PI;
    }
    let c = 1.0 / (u * u * u);
    let near = (a - om).abs();
    let first = if near > 0.0 { (1.0 - c) * near.ln() } else { 0.0 };
    (first + (1.0 + c) * (a + om).ln() - 2.0 / (u * u) + 2.0) / PI
}

impl Mesh {
    pub fn new(anchors: Vec<f64>, w: &[f64], tails: bool) -> Self {
        let ma = anchors.len();
        let mi = ma - 2;
        let mut cm = Vec::with_capacity(mi);
        let mut c0 = Vec::with_capacity(mi);
        let mut cp = Vec::with_capacity(mi);
        for j in 0..mi {
            let (wm, wj, wp) = (anchors[j], anchors[j + 1], anchors[j + 2]);
            // Con colas, la primera y la última ancla no bajan en rampa hasta el nodo de borde.
            let up = if tails && j == 0 { 0.0 } else { 1.0 / (PI * (wj - wm).max(1e-300)) };
            let down = if tails && j + 1 == mi { 0.0 } else { 1.0 / (PI * (wp - wj).max(1e-300)) };
            cm.push(up);
            c0.push(-(up + down));
            cp.push(down);
        }
        let n = w.len();
        let mut left = Vec::with_capacity(n);
        let mut right = Vec::with_capacity(n);
        let mut b_left = Vec::with_capacity(n);
        let mut b_right = Vec::with_capacity(n);
        let mut segs = Vec::with_capacity(n);
        let mut seg = 0usize;
        for &om in w {
            while seg + 2 < ma && anchors[seg + 1] <= om {
                seg += 1;
            }
            let span = anchors[seg + 1] - anchors[seg];
            let t = if span > 0.0 { ((om - anchors[seg]) / span).clamp(0.0, 1.0) } else { 0.0 };
            // El ancla `a` es el parámetro a − 1. Las de borde (0 y ma − 1) valen cero.
            let (l, mut bl) = if seg >= 1 { (seg - 1, 1.0 - t) } else { (0, 0.0) };
            let (r, mut br) = if seg + 1 <= ma - 2 { (seg, t) } else { (mi - 1, 0.0) };
            if tails && seg == 0 {
                br = tail_low_shape(om, anchors[1]);
            }
            if tails && seg + 2 == ma && ma > 2 {
                bl = tail_high_shape(om, anchors[ma - 2]);
            }
            left.push(l);
            right.push(r);
            b_left.push(bl);
            b_right.push(br);
            segs.push(seg);
        }
        Mesh { anchors, cm, c0, cp, left, right, b_left, b_right, seg: segs, tails }
    }

    pub fn interior(&self) -> usize {
        self.anchors.len() - 2
    }

    /// ε₂ variacional en ω.
    fn eps2_at(&self, om: f64, p: &[f64]) -> f64 {
        let ma = self.anchors.len();
        if self.tails {
            if om <= self.anchors[1] {
                return p[0] * tail_low_shape(om, self.anchors[1]);
            }
            if om >= self.anchors[ma - 2] {
                return p[ma - 3] * tail_high_shape(om, self.anchors[ma - 2]);
            }
        }
        if om <= self.anchors[0] || om >= self.anchors[ma - 1] {
            return 0.0;
        }
        let mut lo = 0usize;
        let mut hi = ma - 1;
        while hi - lo > 1 {
            let mid = (lo + hi) >> 1;
            if self.anchors[mid] <= om {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let value = |a: usize| if a == 0 || a == ma - 1 { 0.0 } else { p[a - 1] };
        let span = self.anchors[hi] - self.anchors[lo];
        let t = if span > 0.0 { (om - self.anchors[lo]) / span } else { 0.0 };
        value(lo) + t * (value(hi) - value(lo))
    }

    /// Coeficiente de g(ω, ancla k) en Σ Aⱼ Kⱼ(ω).
    fn kk_coefficients(&self, p: &[f64]) -> Vec<f64> {
        let ma = self.anchors.len();
        let mi = ma - 2;
        let mut coef = vec![0.0; ma];
        for j in 0..mi {
            let a = p[j];
            if a == 0.0 {
                continue;
            }
            coef[j] += a * self.cm[j];
            coef[j + 1] += a * self.c0[j];
            coef[j + 2] += a * self.cp[j];
        }
        coef
    }

    /// Núcleos de las dos colas en ω: multiplican a la primera y a la última altura.
    #[inline]
    fn tail_kernels(&self, om: f64) -> (f64, f64) {
        if !self.tails {
            return (0.0, 0.0);
        }
        let ma = self.anchors.len();
        (tail_low_kernel(om, self.anchors[1]), tail_high_kernel(om, self.anchors[ma - 2]))
    }
}

#[inline(always)]
fn kuzmenko_g(x: f64, y: f64) -> f64 {
    let sp = x + y;
    let sm = x - y;
    let a = if sp == 0.0 { 0.0 } else { sp * sp.abs().ln() };
    let b = if sm == 0.0 { 0.0 } else { sm * sm.abs().ln() };
    a - b
}

/// Σ aⱼ Kⱼ(ω) en cada ω pedida: la parte de ε₁ que sale de la malla triangular (y de sus
/// colas, si las hay). `a` lleva una altura por ancla, con los bordes en cero.
pub(crate) fn kk_values(anchors: &[f64], a: &[f64], omegas: &[f64], tails: bool) -> Vec<f64> {
    let ma = anchors.len();
    let mut coef = vec![0.0; ma];
    for j in 1..ma.saturating_sub(1) {
        if a[j] == 0.0 {
            continue;
        }
        let (wm, wj, wp) = (anchors[j - 1], anchors[j], anchors[j + 1]);
        let up = if tails && j == 1 { 0.0 } else { 1.0 / (PI * (wj - wm).max(1e-300)) };
        let down = if tails && j + 2 == ma { 0.0 } else { 1.0 / (PI * (wp - wj).max(1e-300)) };
        coef[j - 1] += a[j] * up;
        coef[j] -= a[j] * (up + down);
        coef[j + 1] += a[j] * down;
    }
    let tails = tails && ma > 2;
    let mut out = vec![0.0; omegas.len()];
    let workers = worker_count((omegas.len() * ma) as f64 * 8.0);
    let chunk = omegas.len().div_ceil(workers).max(1);
    let job = |start: usize, part: &mut [f64]| {
        for (k, slot) in part.iter_mut().enumerate() {
            let om = omegas[start + k];
            let mut s = 0.0;
            for (c, &anchor) in coef.iter().zip(anchors) {
                if *c != 0.0 {
                    s += c * kuzmenko_g(om, anchor);
                }
            }
            if tails {
                s += a[1] * tail_low_kernel(om, anchors[1]) + a[ma - 2] * tail_high_kernel(om, anchors[ma - 2]);
            }
            *slot = s;
        }
    };
    if workers <= 1 {
        job(0, &mut out);
    } else {
        std::thread::scope(|scope| {
            for (t, part) in out.chunks_mut(chunk).enumerate() {
                let job = &job;
                scope.spawn(move || job(t * chunk, part));
            }
        });
    }
    out
}

/// ε₂ variacional en una ω cualquiera, dentro o fuera de la malla.
pub(crate) fn eps2_value(anchors: &[f64], a: &[f64], om: f64, tails: bool) -> f64 {
    let ma = anchors.len();
    if ma < 3 {
        return 0.0;
    }
    if tails {
        if om <= anchors[1] {
            return a[1] * tail_low_shape(om, anchors[1]);
        }
        if om >= anchors[ma - 2] {
            return a[ma - 2] * tail_high_shape(om, anchors[ma - 2]);
        }
    }
    if om <= anchors[0] || om >= anchors[ma - 1] {
        return 0.0;
    }
    let hi = anchors.partition_point(|v| *v <= om).clamp(1, ma - 1);
    let lo = hi - 1;
    let span = anchors[hi] - anchors[lo];
    let t = if span > 0.0 { (om - anchors[lo]) / span } else { 0.0 };
    a[lo] + t * (a[hi] - a[lo])
}

/// Valores del modelo en cada dato.
#[derive(Clone, Default)]
pub(crate) struct Points {
    pub q: Vec<f64>,
    pub e1: Vec<f64>,
    pub e2: Vec<f64>,
}

impl Points {
    fn new(n: usize) -> Self {
        Points { q: vec![0.0; n], e1: vec![0.0; n], e2: vec![0.0; n] }
    }
}

impl Problem<'_> {
    fn ndat(&self) -> usize {
        self.w.len()
    }

    fn mfit(&self, mesh: &Mesh) -> usize {
        mesh.interior() + usize::from(self.fit_eps_inf)
    }

    fn eps2(&self, mesh: &Mesh, p: &[f64], i: usize) -> f64 {
        self.e2_fixed[i] + mesh.b_left[i] * p[mesh.left[i]] + mesh.b_right[i] * p[mesh.right[i]]
    }

    /// Suavidad sobre tripletas de anclas interiores: valor, y si se pide, α y β.
    fn smooth_terms(&self, mi: usize, p: &[f64], mut sink: Option<(&mut [f64], usize, &mut [f64])>) -> f64 {
        if !(self.smoothness > 0.0) || mi < 3 {
            return 0.0;
        }
        let mut total = 0.0;
        for j in 1..mi - 1 {
            let d2 = p[j - 1] - 2.0 * p[j] + p[j + 1];
            total += self.smoothness * d2 * d2;
            if let Some((h, m, b)) = sink.as_mut() {
                let grad = self.smoothness * d2;
                let pairs = [(j - 1, 1.0), (j, -2.0), (j + 1, 1.0)];
                for (x, &(ja, ca)) in pairs.iter().enumerate() {
                    b[ja] -= grad * ca;
                    for &(jb, cb) in &pairs[..=x] {
                        h[ja * *m + jb] += self.smoothness * ca * cb;
                    }
                }
            }
        }
        total
    }

    /// Solo el objetivo (χ² más castigos) y los valores del modelo. Cuesta N·M logaritmos.
    fn objective(&self, mesh: &Mesh, p: &[f64], pts: &mut Points, threads: usize) -> (f64, f64) {
        let n = self.ndat();
        let mi = mesh.interior();
        let coef = mesh.kk_coefficients(p);
        let base = p[mi];
        let workers = if threads <= 1 { 1 } else { worker_count((n * mesh.anchors.len()) as f64 * 8.0).min(threads) };
        let chunk = n.div_ceil(workers).max(1);
        let run = |start: usize, q: &mut [f64], e1: &mut [f64], e2: &mut [f64]| -> (f64, f64) {
            let mut data = 0.0;
            let mut pen = 0.0;
            for k in 0..q.len() {
                let i = start + k;
                let om = self.w[i];
                let mut s = 0.0;
                for (c, &a) in coef.iter().zip(&mesh.anchors) {
                    if *c != 0.0 {
                        s += c * kuzmenko_g(om, a);
                    }
                }
                if mesh.tails {
                    let (lo, hi) = mesh.tail_kernels(om);
                    s += p[0] * lo + p[mi - 1] * hi;
                }
                let v1 = base + self.e1_fixed[i] + s;
                let v2 = self.eps2(mesh, p, i);
                let (qi, _, _) = quantity_grad(self.measure, v1, v2, self.nu[i], self.d_cm);
                let dy = self.y[i] - qi;
                data += dy * dy * self.inv_var[i];
                if self.negativity > 0.0 && v2 < 0.0 {
                    pen += self.negativity * v2 * v2;
                }
                q[k] = qi;
                e1[k] = v1;
                e2[k] = v2;
            }
            (data, pen)
        };
        let (data, pen) = if workers <= 1 {
            run(0, &mut pts.q, &mut pts.e1, &mut pts.e2)
        } else {
            std::thread::scope(|scope| {
                let mut handles = Vec::new();
                let parts = pts.q.chunks_mut(chunk).zip(pts.e1.chunks_mut(chunk)).zip(pts.e2.chunks_mut(chunk));
                for (t, ((q, e1), e2)) in parts.enumerate() {
                    let run = &run;
                    handles.push(scope.spawn(move || run(t * chunk, q, e1, e2)));
                }
                handles.into_iter().fold((0.0, 0.0), |acc, h| {
                    let (d, p) = h.join().unwrap_or((f64::NAN, 0.0));
                    (acc.0 + d, acc.1 + p)
                })
            })
        };
        let smooth = self.smooth_terms(mi, p, None);
        (data + pen + smooth, data)
    }

    /// Filas del jacobiano (ya multiplicadas por √peso) y valores del modelo, datos [i0, i0 + filas).
    #[allow(clippy::too_many_arguments)]
    fn jacobian_rows(
        &self,
        mesh: &Mesh,
        p: &[f64],
        i0: usize,
        jb: &mut [f64],
        m: usize,
        q: &mut [f64],
        e1: &mut [f64],
        e2: &mut [f64],
        g: &mut [f64],
    ) {
        let mi = mesh.interior();
        let base = p[mi];
        for (k, row) in jb.chunks_exact_mut(m).enumerate() {
            let i = i0 + k;
            let om = self.w[i];
            for (gk, &a) in g.iter_mut().zip(&mesh.anchors) {
                *gk = kuzmenko_g(om, a);
            }
            {
                let kern = &mut row[..mi];
                let (g0, g1, g2) = (&g[..mi], &g[1..mi + 1], &g[2..mi + 2]);
                for j in 0..mi {
                    kern[j] = mesh.cm[j] * g0[j] + mesh.c0[j] * g1[j] + mesh.cp[j] * g2[j];
                }
                if mesh.tails {
                    let (lo, hi) = mesh.tail_kernels(om);
                    kern[0] += lo;
                    kern[mi - 1] += hi;
                }
            }
            let v1 = base + self.e1_fixed[i] + dot(&row[..mi], &p[..mi]);
            let v2 = self.eps2(mesh, p, i);
            let (qi, d1, d2) = quantity_grad(self.measure, v1, v2, self.nu[i], self.d_cm);
            let sw = self.inv_var[i].sqrt();
            let s1 = d1 * sw;
            for v in row[..mi].iter_mut() {
                *v *= s1;
            }
            row[mesh.left[i]] += d2 * sw * mesh.b_left[i];
            row[mesh.right[i]] += d2 * sw * mesh.b_right[i];
            if self.fit_eps_inf {
                row[mi] = s1;
            }
            q[k] = qi;
            e1[k] = v1;
            e2[k] = v2;
        }
    }

    /// α (triángulo inferior de `h`), β y el objetivo en `p`. Con `penalties` en falso queda
    /// la curvatura de los datos solos, que es la que da la barra de confianza.
    #[allow(clippy::too_many_arguments)]
    fn assemble(
        &self,
        mesh: &Mesh,
        p: &[f64],
        h: &mut [f64],
        b: &mut [f64],
        pts: &mut Points,
        penalties: bool,
        threads: usize,
        cancel: Option<&AtomicBool>,
    ) -> Result<(f64, f64), String> {
        let n = self.ndat();
        let mi = mesh.interior();
        let ma = mesh.anchors.len();
        let m = self.mfit(mesh);
        h[..m * m].fill(0.0);
        b[..m].fill(0.0);
        let workers = worker_count(n as f64 * (m * m) as f64).min(threads.max(1));
        // El bloque de filas cabe en caché: unas 128 mil celdas.
        let nb = ((131_072 / m.max(1)).clamp(8, 256) / 4 * 4).max(4);
        let mut jb = vec![0.0; nb * m];
        let mut data = 0.0;
        let mut pen = 0.0;
        let cuts = triangular_cuts(0, m, workers);
        let mut i0 = 0;
        while i0 < n {
            if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
                return Err("cancelado".into());
            }
            let rows = nb.min(n - i0);
            let block = &mut jb[..rows * m];
            let (q, e1, e2) = (&mut pts.q[i0..i0 + rows], &mut pts.e1[i0..i0 + rows], &mut pts.e2[i0..i0 + rows]);
            if workers <= 1 || rows < 2 * workers {
                let mut g = vec![0.0; ma];
                self.jacobian_rows(mesh, p, i0, block, m, q, e1, e2, &mut g);
            } else {
                let chunk = rows.div_ceil(workers);
                std::thread::scope(|scope| {
                    let parts = block.chunks_mut(chunk * m).zip(q.chunks_mut(chunk)).zip(e1.chunks_mut(chunk)).zip(e2.chunks_mut(chunk));
                    for (t, (((jrows, q), e1), e2)) in parts.enumerate() {
                        scope.spawn(move || {
                            let mut g = vec![0.0; ma];
                            self.jacobian_rows(mesh, p, i0 + t * chunk, jrows, m, q, e1, e2, &mut g);
                        });
                    }
                });
            }
            for (k, row) in block.chunks_exact(m).enumerate() {
                let i = i0 + k;
                let sw = self.inv_var[i].sqrt();
                let r = (self.y[i] - pts.q[i]) * sw;
                data += r * r;
                for (bj, v) in b[..m].iter_mut().zip(row) {
                    *bj += r * v;
                }
            }
            let block = &jb[..rows * m];
            par_rows(&mut h[..m * m], m, &cuts, |j0, hs| syrk(hs, m, j0, block, rows));
            i0 += rows;
        }
        if penalties {
            if self.negativity > 0.0 {
                for i in 0..n {
                    let v2 = pts.e2[i];
                    if v2 < 0.0 {
                        pen += self.negativity * v2 * v2;
                        let (l, r) = (mesh.left[i], mesh.right[i]);
                        let (bl, br) = (mesh.b_left[i], mesh.b_right[i]);
                        b[l] -= self.negativity * v2 * bl;
                        h[l * m + l] += self.negativity * bl * bl;
                        if r != l {
                            b[r] -= self.negativity * v2 * br;
                            h[r * m + r] += self.negativity * br * br;
                            let (hi, lo) = if r > l { (r, l) } else { (l, r) };
                            h[hi * m + lo] += self.negativity * bl * br;
                        } else {
                            b[l] -= self.negativity * v2 * br;
                            h[l * m + l] += self.negativity * (2.0 * bl * br + br * br);
                        }
                    }
                }
            }
            pen += self.smooth_terms(mi, p, Some((h, m, b)));
        }
        Ok((data + pen, data))
    }
}

/// Lo que el ajuste devuelve a `run_fit`.
pub(crate) struct Solution {
    pub anchors: Vec<f64>,
    /// Alturas en cada ancla (los bordes valen cero) y, al final, ε∞.
    pub a: Vec<f64>,
    pub points: Points,
    pub iters_used: usize,
    pub converged: bool,
    pub stalled: bool,
    pub mesh_used: MeshMode,
    pub solver_used: Solver,
    pub candidates: usize,
    pub levels: usize,
    pub notes: Vec<String>,
    pub errors: Option<Errors>,
}

/// Barras de confianza desde la covarianza de los datos.
pub(crate) struct Errors {
    /// ±1σ de ε₂ en cada dato.
    pub eps2: Vec<f64>,
    /// ±1σ de la magnitud medida en cada dato.
    pub q: Vec<f64>,
    /// ±1σ de cada altura interior.
    pub coef: Vec<f64>,
    pub eps_inf: Option<f64>,
}

pub(crate) struct Config<'a> {
    pub mesh: MeshMode,
    pub solver: Solver,
    pub iter_max: usize,
    pub cancel: Option<&'a AtomicBool>,
    pub progress_step: Option<&'a AtomicUsize>,
    pub progress_curve: &'a Option<Arc<Mutex<Option<FitProgressCurve>>>>,
    /// Posiciones candidatas para las anclas interiores, en orden creciente.
    pub candidates: &'a [f64],
    pub edge_lo: f64,
    pub edge_hi: f64,
    /// ε₂ sigue fuera del rango con colas suaves en vez de caer a cero en los nodos de borde.
    pub tails: bool,
    pub want_errors: bool,
}

struct Workspace {
    h: Vec<f64>,
    work: Vec<f64>,
    b: Vec<f64>,
    threads: usize,
}

impl Workspace {
    fn ensure(&mut self, m: usize) {
        if self.h.len() < m * m {
            self.h.resize(m * m, 0.0);
            self.work.resize(m * m, 0.0);
        }
        if self.b.len() < m {
            self.b.resize(m, 0.0);
        }
    }

    /// Resuelve (α + λ·diag α)·x = β.
    fn step(&mut self, m: usize, lam: f64, solver: Solver) -> Option<Vec<f64>> {
        let (h, work) = (&self.h[..m * m], &mut self.work[..m * m]);
        let top = (0..m).map(|j| h[j * m + j]).fold(0.0, f64::max);
        if !(top > 0.0) || !top.is_finite() {
            return None;
        }
        let mut x = self.b[..m].to_vec();
        match solver {
            Solver::GaussJordan => {
                for j in 0..m {
                    for k in 0..=j {
                        let v = h[j * m + k];
                        work[j * m + k] = v;
                        work[k * m + j] = v;
                    }
                }
            }
            _ => {
                for j in 0..m {
                    work[j * m..j * m + j + 1].copy_from_slice(&h[j * m..j * m + j + 1]);
                }
            }
        }
        for j in 0..m {
            // Un ancla sin sensibilidad dejaría un cero en la diagonal: el suelo relativo la fija.
            work[j * m + j] = h[j * m + j] * (1.0 + lam) + top * 1e-14;
        }
        let ok = match solver {
            Solver::GaussJordan => gauss_jordan_solve(work, m, &mut x, self.threads),
            _ => cholesky_solve(work, m, &mut x, self.threads),
        };
        ok.then_some(x)
    }
}

fn pick_solver(requested: Solver, m: usize) -> Solver {
    match requested {
        Solver::Auto => {
            if m <= AUTO_GAUSS_JORDAN_MAX {
                Solver::GaussJordan
            } else {
                Solver::Cholesky
            }
        }
        other => other,
    }
}

fn publish(
    slot: &Option<Arc<Mutex<Option<FitProgressCurve>>>>,
    problem: &Problem<'_>,
    mesh: &Mesh,
    p: &[f64],
    pts: &Points,
    chisq: f64,
) {
    let Some(slot) = slot else { return };
    let n = problem.ndat();
    if n < 2 {
        return;
    }
    // La vista en vivo conserva, en cada tramo, el dato más alejado de la recta entre vecinos:
    // así un reststrahlen estrecho no desaparece al reducir.
    let cap = 480usize;
    let stride = n.div_ceil(cap).max(1);
    let mut x = Vec::with_capacity(cap + 2);
    let mut data = Vec::with_capacity(cap + 2);
    let mut model = Vec::with_capacity(cap + 2);
    let mut resid2 = 0.0;
    for i in 0..n {
        let dy = problem.y[i] - pts.q[i];
        resid2 += dy * dy;
    }
    let mut start = 0;
    while start < n {
        let end = (start + stride).min(n);
        let mut lo = start;
        let mut hi = start;
        for i in start..end {
            if problem.y[i] < problem.y[lo] {
                lo = i;
            }
            if problem.y[i] > problem.y[hi] {
                hi = i;
            }
        }
        let (first, second) = if lo <= hi { (lo, hi) } else { (hi, lo) };
        for i in [first, second] {
            if x.last().copied() != Some(problem.w[i]) {
                x.push(problem.w[i]);
                data.push(problem.y[i]);
                model.push(pts.q[i]);
            }
        }
        start = end;
    }
    let mi = mesh.interior();
    let mesh_cap = 320usize;
    let mesh_stride = mi.div_ceil(mesh_cap).max(1);
    let mut mesh_x = Vec::with_capacity(mi / mesh_stride + 4);
    let mut mesh_y = Vec::with_capacity(mi / mesh_stride + 4);
    if !mesh.tails {
        mesh_x.push(mesh.anchors[0]);
        mesh_y.push(0.0);
    }
    let mut j = 0;
    while j < mi {
        // En cada tramo se dibuja el ancla más alta, para no perder los picos de ε₂.
        let end = (j + mesh_stride).min(mi);
        let mut best = j;
        for k in j..end {
            if p[k] > p[best] {
                best = k;
            }
        }
        mesh_x.push(mesh.anchors[best + 1]);
        mesh_y.push(p[best].max(0.0));
        j = end;
    }
    if !mesh.tails {
        mesh_x.push(mesh.anchors[mi + 1]);
        mesh_y.push(0.0);
    }
    if let Ok(mut guard) = slot.lock() {
        *guard = Some(FitProgressCurve {
            x,
            data,
            model,
            mesh_x,
            mesh_y,
            chisq: Some(chisq),
            resid_rms: Some((resid2 / n as f64).sqrt()),
        });
    }
}

struct Run<'a> {
    problem: &'a Problem<'a>,
    cfg: &'a Config<'a>,
    ws: Workspace,
    iters: usize,
    solver_used: Solver,
    /// χ² de datos por debajo del cual el ajuste ya es exacto a efectos prácticos.
    floor: f64,
    /// σ del ruido en cada dato: la declarada o la estimada de la propia medida.
    sigma: Vec<f64>,
    scratch: Vec<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum End {
    Converged,
    /// Se agotaron los pasos de este nivel.
    Steps,
    /// λ creció sin que el objetivo bajara.
    Stalled,
}

impl Run<'_> {
    /// Mediana y máximo de |yᵢ − modeloᵢ|/σᵢ, escalados para valer ~1 con ruido gaussiano.
    /// La mediana evita que un flanco abrupto (σ sobreestimada) mande en la parada; el máximo
    /// evita parar cuando el grueso ya está en el ruido pero un mínimo LO u otro rasgo local
    /// sigue con residuo enorme.
    fn noise_stats(&mut self, pts: &Points) -> (f64, f64) {
        let problem = self.problem;
        self.scratch.clear();
        let mut max_r = 0.0_f64;
        for ((y, q), s) in problem.y.iter().zip(&pts.q).zip(&self.sigma) {
            let r = (y - q).abs();
            let u = if *s > 0.0 {
                r / s
            } else if r == 0.0 {
                0.0
            } else {
                f64::INFINITY
            };
            if u > max_r {
                max_r = u;
            }
            self.scratch.push(u);
        }
        let med = median_in_place(&mut self.scratch) / 0.674_489_75;
        (med, max_r / 0.674_489_75)
    }

    /// Levenberg–Marquardt (Numerical Recipes §15.5) sobre una malla fija.
    ///
    /// En la malla final (`last`) la parada sigue el principio de discrepancia: con el residuo
    /// ya dentro del ruido, seguir solo copia el ruido en ε₂.
    fn levenberg(&mut self, mesh: &Mesh, p: &mut Vec<f64>, pts: &mut Points, max_steps: usize, tol: f64, last: bool) -> Result<(End, f64, f64), String> {
        let problem = self.problem;
        let mi = mesh.interior();
        let m = problem.mfit(mesh);
        self.ws.ensure(m);
        let threads = self.ws.threads;
        let solver = pick_solver(self.cfg.solver, m);
        self.solver_used = solver;
        let (mut chi, mut data) = problem.assemble(mesh, p, &mut self.ws.h, &mut self.ws.b, pts, true, threads, self.cfg.cancel)?;
        if !chi.is_finite() {
            return Err("el χ² inicial no es finito. Revisa el espectro.".into());
        }
        publish(self.cfg.progress_curve, problem, mesh, p, pts, data);
        let mut lam = 1e-3;
        let mut trial = p.clone();
        let mut trial_pts = Points::new(problem.ndat());
        let mut end = End::Steps;
        let mut steps = 0usize;
        let mut flat = 0usize;
        let mut accepted = 0usize;
        let mut failed_last = false;
        // Un cambio relativo de χ² menor que su propia dispersión estadística, √(2/N), no dice nada.
        let noise_tol = (0.5 * (2.0 / problem.ndat().max(1) as f64).sqrt()).max(tol);
        while steps < max_steps {
            if data <= self.floor {
                end = End::Converged;
                break;
            }
            if self.cfg.cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
                return Err("cancelado".into());
            }
            steps += 1;
            self.iters += 1;
            if let Some(step) = self.cfg.progress_step {
                step.store(self.iters, Ordering::Relaxed);
            }
            let Some(dx) = self.ws.step(m, lam, solver) else {
                lam *= 10.0;
                if lam > 1e12 {
                    end = End::Stalled;
                    break;
                }
                continue;
            };
            for j in 0..mi {
                trial[j] = p[j] + dx[j];
            }
            trial[mi] = if problem.fit_eps_inf { p[mi] + dx[mi] } else { p[mi] };
            if let Some((lo, hi)) = problem.eps_inf_bounds {
                if problem.fit_eps_inf {
                    trial[mi] = trial[mi].clamp(lo, hi);
                }
            }
            let (chi_try, data_try) = problem.objective(mesh, &trial, &mut trial_pts, threads);
            if chi_try.is_finite() && chi_try <= chi {
                let gain = chi - chi_try;
                let (ratio, max_ratio) = if last { self.noise_stats(&trial_pts) } else { (f64::INFINITY, f64::INFINITY) };
                let near_noise = ratio <= 2.0 && max_ratio <= 8.0;
                let small = gain <= self.floor.max(if near_noise { noise_tol } else { tol } * chi);
                std::mem::swap(p, &mut trial);
                std::mem::swap(pts, &mut trial_pts);
                chi = chi_try;
                data = data_try;
                accepted += 1;
                // Tras un rechazo, λ se queda donde funcionó: bajarla enseguida repite el rechazo.
                if !failed_last {
                    lam = (lam * 0.1).max(1e-9);
                }
                failed_last = false;
                publish(self.cfg.progress_curve, problem, mesh, p, pts, data);
                if small {
                    flat += 1;
                } else {
                    flat = 0;
                }
                if flat >= 2 || data <= self.floor || (accepted >= 1 && ratio <= 1.0 && max_ratio <= 5.0) {
                    end = End::Converged;
                    break;
                }
                if steps < max_steps {
                    let (c, d) = problem.assemble(mesh, p, &mut self.ws.h, &mut self.ws.b, pts, true, threads, self.cfg.cancel)?;
                    chi = c;
                    data = d;
                }
            } else {
                // Un paso que no baja el objetivo, y casi no lo mueve, es la misma parada.
                let nearly = chi_try.is_finite() && (chi_try - chi) <= self.floor.max(tol * chi);
                failed_last = true;
                lam *= 10.0;
                if nearly && lam >= 1.0 {
                    end = End::Converged;
                    break;
                }
                if lam > 1e10 {
                    end = End::Stalled;
                    break;
                }
            }
        }
        Ok((end, chi, data))
    }
}

/// Índices de las candidatas que forman la malla de paso `stride`. Siempre entran la primera y la última.
fn strided(count: usize, stride: usize) -> Vec<usize> {
    let mut out: Vec<usize> = (0..count).step_by(stride.max(1)).collect();
    if out.last().copied() != Some(count - 1) {
        out.push(count - 1);
    }
    out
}

fn build_mesh(cfg: &Config<'_>, picks: &[usize], w: &[f64]) -> Mesh {
    let mut anchors = Vec::with_capacity(picks.len() + 2);
    anchors.push(cfg.edge_lo);
    anchors.extend(picks.iter().map(|&i| cfg.candidates[i]));
    anchors.push(cfg.edge_hi);
    Mesh::new(anchors, w, cfg.tails)
}

/// Lleva las alturas de una malla a otra que la contiene: ε₂ no cambia.
fn prolong(old: &Mesh, p: &[f64], new: &Mesh) -> Vec<f64> {
    let mi = new.interior();
    let mut out = Vec::with_capacity(mi + 1);
    for j in 0..mi {
        out.push(old.eps2_at(new.anchors[j + 1], p));
    }
    out.push(p[old.interior()]);
    out
}

/// Parámetros de partida en una malla: ε₂ de la semilla en cada ancla interior y ε∞ dada.
fn seed_params(mesh: &Mesh, seed: &dyn Fn(f64) -> f64, eps_inf: f64, bounds: Option<(f64, f64)>) -> Vec<f64> {
    let mi = mesh.interior();
    let mut p: Vec<f64> = (0..mi).map(|j| seed(mesh.anchors[j + 1])).collect();
    let mut e = eps_inf;
    if let Some((lo, hi)) = bounds {
        e = e.clamp(lo, hi);
    }
    p.push(e);
    p
}

/// Si la semilla reproduce mejor los datos que `p` (p. ej. tras una prolongación que aplastó
/// un mínimo LO), sustituye `p` por la semilla. La continuación gruesa→fina no puede inventar
/// rasgos más estrechos que su paso; sin este rescate el LM fino arranca ya en un mínimo malo.
fn take_seed_if_better(
    problem: &Problem<'_>,
    mesh: &Mesh,
    p: &mut Vec<f64>,
    pts: &mut Points,
    seed: &dyn Fn(f64) -> f64,
    eps_inf: f64,
    threads: usize,
) {
    let seeded = seed_params(mesh, seed, eps_inf, problem.eps_inf_bounds.filter(|_| problem.fit_eps_inf));
    let mut seed_pts = Points::new(problem.ndat());
    let (_, data_seed) = problem.objective(mesh, &seeded, &mut seed_pts, threads);
    let (_, data_cur) = problem.objective(mesh, p, pts, threads);
    if data_seed.is_finite() && data_seed < data_cur {
        *p = seeded;
        *pts = seed_pts;
    }
}

/// |y_i − cúbica por sus cuatro vecinos|, normalizado para que su escala sea la σ del ruido.
/// La cúbica absorbe la curvatura de un pico real en cualquier malla, uniforme o no, de modo
/// que lo que queda es ruido salvo en los flancos más abruptos.
fn neighbour_deviation(w: &[f64], y: &[f64]) -> Vec<f64> {
    let n = y.len();
    if n < 7 {
        return Vec::new();
    }
    (2..n - 2)
        .map(|i| {
            let nodes = [i - 2, i - 1, i + 1, i + 2];
            let mut fit = 0.0;
            let mut norm = 1.0;
            for &a in &nodes {
                let mut l = 1.0;
                for &b in &nodes {
                    if a != b {
                        let den = w[a] - w[b];
                        l *= if den != 0.0 { (w[i] - w[b]) / den } else { 0.0 };
                    }
                }
                fit += l * y[a];
                norm += l * l;
            }
            (y[i] - fit).abs() / norm.sqrt()
        })
        .collect()
}

fn median_in_place(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mid = values.len() / 2;
    values.select_nth_unstable_by(mid, |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values[mid]
}

/// Ruido de la medida: mediana de la desviación respecto a los vecinos, a prueba de picos reales.
fn noise_level(w: &[f64], y: &[f64]) -> f64 {
    let mut dev = neighbour_deviation(w, y);
    1.4826 * median_in_place(&mut dev)
}

/// Ruido en cada dato: la misma mediana, en una ventana de ±`NOISE_WINDOW` puntos. Un
/// espectrómetro no tiene el mismo ruido en todo el rango.
fn noise_profile(w: &[f64], y: &[f64]) -> Vec<f64> {
    let n = y.len();
    let dev = neighbour_deviation(w, y);
    if dev.is_empty() {
        return vec![0.0; n];
    }
    let mut buf = Vec::with_capacity(2 * NOISE_WINDOW + 1);
    (0..n)
        .map(|i| {
            // dev[k] corresponde al dato k + 2.
            let centre = i.saturating_sub(2).min(dev.len() - 1);
            let lo = centre.saturating_sub(NOISE_WINDOW);
            let hi = (centre + NOISE_WINDOW + 1).min(dev.len());
            buf.clear();
            buf.extend_from_slice(&dev[lo..hi]);
            1.4826 * median_in_place(&mut buf)
        })
        .collect()
}

/// Ajusta la función variacional a la medida y devuelve la malla, las alturas y el modelo.
pub(crate) fn solve(problem: &Problem<'_>, cfg: &Config<'_>, seed: &dyn Fn(f64) -> f64, eps_inf: f64) -> Result<Solution, String> {
    let n = problem.ndat();
    let count = cfg.candidates.len();
    if count == 0 {
        return Err("hacen falta al menos 3 anclas".into());
    }
    let cap = anchor_cap(n);
    let mut notes = Vec::new();
    let mut mode = cfg.mesh;
    if mode == MeshMode::Auto {
        mode = if count <= cap { MeshMode::Full } else { MeshMode::Adaptive };
    }
    let target = match mode {
        MeshMode::Half => 2usize,
        _ => 1usize,
    };
    if mode != MeshMode::Adaptive && count.div_ceil(target) > cap {
        notes.push(format!(
            "La malla pedida tendría {} anclas y el presupuesto de cómputo para {n} puntos admite {cap}: se usa la malla adaptativa.{}",
            count.div_ceil(target),
            if cfg!(target_arch = "wasm32") { " En el servidor el presupuesto es veinte veces mayor." } else { "" }
        ));
        mode = MeshMode::Adaptive;
    }
    let threads = worker_count(f64::INFINITY);
    // En las mallas gruesas ε∞ se queda en su valor de partida. Una malla gruesa no puede
    // seguir la medida, y si ε∞ está libre absorbe ese defecto junto con ε₂ donde la muestra es
    // casi transparente; la medida apenas distingue esa combinación y la malla fina ya no la
    // deshace. ε∞ se libera en la malla final, o antes si el ajuste grueso queda lejos de los
    // datos, que es señal de que la partida de ε∞ era mala.
    let frozen = Problem { fit_eps_inf: false, ..*problem };
    let mut freeze = problem.fit_eps_inf;
    let (y_lo, y_hi) = problem.y.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |acc, v| (acc.0.min(*v), acc.1.max(*v)));
    let y_range = (y_hi - y_lo).max(1e-300);
    let poor_fit = |pts: &Points| {
        let sq: f64 = problem.y.iter().zip(&pts.q).map(|(y, q)| (y - q) * (y - q)).sum();
        (sq / n as f64).sqrt() > 0.02 * y_range
    };
    let scale: f64 = problem.y.iter().zip(problem.inv_var).map(|(y, w)| y * y * w).sum::<f64>().max(1e-300);
    let mut run = Run {
        problem,
        cfg,
        ws: Workspace { h: Vec::new(), work: Vec::new(), b: Vec::new(), threads },
        iters: 0,
        solver_used: pick_solver(cfg.solver, count),
        floor: 1e-12 * scale,
        sigma: if problem.errors_given {
            problem.inv_var.iter().map(|w| if *w > 0.0 { 1.0 / w.sqrt() } else { 0.0 }).collect()
        } else {
            noise_profile(problem.w, problem.y)
        },
        scratch: Vec::with_capacity(n),
    };
    let iter_max = cfg.iter_max.max(1);

    // Nivel más grueso: se dobla el paso hasta quedar cerca de COARSE_ANCHORS.
    let mut stride = target;
    while count / (stride * 2) >= COARSE_ANCHORS {
        stride *= 2;
    }
    // Con muy pocos pasos disponibles no hay continuación: se va directo a la malla final.
    let mut planned = 1usize;
    {
        let mut s = stride;
        while s > target {
            planned += 1;
            s /= 2;
        }
    }
    while planned > 1 && iter_max < 2 * planned {
        stride /= 2;
        planned -= 1;
    }
    let coarse_steps = COARSE_STEPS.min((iter_max / (2 * planned)).max(1));

    let mut picks = strided(count, stride);
    let mut mesh = build_mesh(cfg, &picks, problem.w);
    let mut p = seed_params(&mesh, seed, eps_inf, problem.eps_inf_bounds.filter(|_| problem.fit_eps_inf));
    let mut pts = Points::new(n);
    let mut levels = 1usize;
    let mut end = End::Steps;
    let final_tol = 1e-3;

    if mode == MeshMode::Adaptive {
        let noise = noise_level(problem.w, problem.y);
        let range = y_range;
        loop {
            let left = iter_max.saturating_sub(run.iters);
            if left == 0 {
                break;
            }
            run.problem = if freeze { &frozen } else { problem };
            let (level_end, _, _) = run.levenberg(&mesh, &mut p, &mut pts, coarse_steps.min(left), 0.02, false)?;
            end = level_end;
            if freeze && poor_fit(&pts) {
                freeze = false;
            }
            if mesh.interior() >= cap {
                break;
            }
            // Un tramo se afina si algún dato dentro queda fuera del ruido.
            let ma = mesh.anchors.len();
            let mut flagged = vec![false; ma - 1];
            let mut any = false;
            for i in 0..n {
                let r = (problem.y[i] - pts.q[i]).abs();
                let limit = (4.0 * noise).max(2e-4 * range);
                // Con errores declarados manda el propio error: tres desviaciones.
                let far = if problem.errors_given { r * problem.inv_var[i].sqrt() > 3.0 } else { r > limit };
                if far {
                    flagged[mesh.seg[i]] = true;
                    any = true;
                }
            }
            if !any {
                break;
            }
            // Los tramos vecinos se afinan también: ε₁ acopla a los que están cerca.
            let grown: Vec<bool> = (0..ma - 1)
                .map(|k| flagged[k] || (k > 0 && flagged[k - 1]) || (k + 2 < ma && flagged[k + 1]))
                .collect();
            let mut next = Vec::with_capacity(picks.len() * 2);
            let mut budget = cap.saturating_sub(picks.len());
            let mut added = 0usize;
            for (k, &pick) in picks.iter().enumerate() {
                next.push(pick);
                if k + 1 < picks.len() && grown[k + 1] && budget > 0 {
                    let gap = picks[k + 1] - pick;
                    if gap >= 2 {
                        next.push(pick + gap / 2);
                        budget -= 1;
                        added += 1;
                    }
                }
            }
            if added == 0 {
                break;
            }
            let finer = build_mesh(cfg, &next, problem.w);
            p = prolong(&mesh, &p, &finer);
            mesh = finer;
            picks = next;
            take_seed_if_better(problem, &mesh, &mut p, &mut pts, seed, eps_inf, threads);
            levels += 1;
        }
        let left = iter_max.saturating_sub(run.iters);
        run.problem = problem;
        take_seed_if_better(problem, &mesh, &mut p, &mut pts, seed, eps_inf, threads);
        if left > 0 {
            let (level_end, _, _) = run.levenberg(&mesh, &mut p, &mut pts, left, final_tol, true)?;
            end = level_end;
        }
    } else {
        loop {
            let last = stride == target;
            let left = iter_max.saturating_sub(run.iters);
            if left == 0 {
                break;
            }
            if last {
                take_seed_if_better(problem, &mesh, &mut p, &mut pts, seed, eps_inf, threads);
            }
            let (steps, tol) = if last { (left, final_tol) } else { (coarse_steps.min(left), 0.02) };
            run.problem = if last || !freeze { problem } else { &frozen };
            let (level_end, _, _) = run.levenberg(&mesh, &mut p, &mut pts, steps, tol, last)?;
            end = level_end;
            if freeze && !last && poor_fit(&pts) {
                freeze = false;
            }
            if last {
                break;
            }
            stride /= 2;
            picks = strided(count, stride);
            let finer = build_mesh(cfg, &picks, problem.w);
            p = prolong(&mesh, &p, &finer);
            mesh = finer;
            take_seed_if_better(problem, &mesh, &mut p, &mut pts, seed, eps_inf, threads);
            levels += 1;
        }
    }

    // Si el tope de pasos cortó antes de evaluar la última malla, los valores se recalculan aquí.
    let (_, data) = problem.objective(&mesh, &p, &mut pts, threads);
    let converged = end == End::Converged || data <= run.floor;
    let stalled = end == End::Stalled && !converged;

    let errors = if cfg.want_errors { covariance(problem, &mesh, &p, &mut run, &mut pts)? } else { None };

    let mi = mesh.interior();
    let mut a = Vec::with_capacity(mi + 3);
    a.push(0.0);
    a.extend_from_slice(&p[..mi]);
    a.push(0.0);
    a.push(p[mi]);
    Ok(Solution {
        anchors: mesh.anchors,
        a,
        points: pts,
        iters_used: run.iters,
        converged,
        stalled,
        mesh_used: mode,
        solver_used: run.solver_used,
        candidates: count,
        levels,
        notes,
        errors,
    })
}

/// Inversa de la curvatura de los datos (sin suavidad ni castigo) y las barras que salen de ella.
fn covariance(problem: &Problem<'_>, mesh: &Mesh, p: &[f64], run: &mut Run<'_>, pts: &mut Points) -> Result<Option<Errors>, String> {
    let n = problem.ndat();
    let mi = mesh.interior();
    let ma = mesh.anchors.len();
    let m = problem.mfit(mesh);
    run.ws.ensure(m);
    let threads = run.ws.threads;
    problem.assemble(mesh, p, &mut run.ws.h, &mut run.ws.b, pts, false, threads, run.cfg.cancel)?;
    let (h, c) = (&run.ws.h[..m * m], &mut run.ws.work[..m * m]);
    for j in 0..m {
        for k in 0..=j {
            let v = h[j * m + k];
            c[j * m + k] = v;
            c[k * m + j] = v;
        }
    }
    let mut rhs = vec![0.0; m];
    if !gauss_jordan(c, m, &mut rhs, threads) {
        return Ok(None);
    }
    let c = &run.ws.work[..m * m];
    let coef: Vec<f64> = (0..mi).map(|j| c[j * m + j].max(0.0).sqrt()).collect();
    let eps_inf = problem.fit_eps_inf.then(|| c[mi * m + mi].max(0.0).sqrt());
    let mut eps2 = Vec::with_capacity(n);
    for i in 0..n {
        let (l, r) = (mesh.left[i], mesh.right[i]);
        let (bl, br) = (mesh.b_left[i], mesh.b_right[i]);
        let v = bl * bl * c[l * m + l] + 2.0 * bl * br * c[l * m + r] + br * br * c[r * m + r];
        eps2.push(v.max(0.0).sqrt());
    }
    // ±1σ de la magnitud medida: √(JᵢCJᵢᵀ), con la fila sin pesar.
    let mut q = vec![0.0; n];
    let workers = worker_count(n as f64 * (m * m) as f64).min(threads);
    let chunk = n.div_ceil(workers).max(1);
    let job = |start: usize, out: &mut [f64]| {
        let mut g = vec![0.0; ma];
        let mut row = vec![0.0; m];
        let mut cr = vec![0.0; m];
        let (mut q1, mut e1, mut e2) = ([0.0], [0.0], [0.0]);
        for (k, slot) in out.iter_mut().enumerate() {
            let i = start + k;
            problem.jacobian_rows(mesh, p, i, &mut row, m, &mut q1, &mut e1, &mut e2, &mut g);
            let sw = problem.inv_var[i].sqrt();
            for (dst, crow) in cr.iter_mut().zip(c.chunks_exact(m)) {
                *dst = dot(crow, &row);
            }
            let v = dot(&cr, &row) / (sw * sw).max(1e-300);
            *slot = v.max(0.0).sqrt();
        }
    };
    if workers <= 1 {
        job(0, &mut q);
    } else {
        std::thread::scope(|scope| {
            for (t, part) in q.chunks_mut(chunk).enumerate() {
                let job = &job;
                scope.spawn(move || job(t * chunk, part));
            }
        });
    }
    Ok(Some(Errors { eps2, q, coef, eps_inf }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spd(n: usize) -> (Vec<f64>, Vec<f64>) {
        // A = BᵀB + n·I con B determinista.
        let mut b = vec![0.0; n * n];
        let mut seed = 12345u64;
        for v in b.iter_mut() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *v = ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5;
        }
        let mut a = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                let mut s = 0.0;
                for k in 0..n {
                    s += b[k * n + i] * b[k * n + j];
                }
                a[i * n + j] = s + if i == j { n as f64 * 0.05 } else { 0.0 };
            }
        }
        let rhs: Vec<f64> = (0..n).map(|i| (i as f64 * 0.37).sin()).collect();
        (a, rhs)
    }

    fn residual(a: &[f64], n: usize, x: &[f64], rhs: &[f64]) -> f64 {
        (0..n)
            .map(|i| {
                let s: f64 = (0..n).map(|j| a[i * n + j] * x[j]).sum();
                (s - rhs[i]).abs()
            })
            .fold(0.0, f64::max)
    }

    #[test]
    fn cholesky_and_gauss_jordan_agree() {
        for n in [1usize, 2, 7, 63, 64, 65, 200, 450] {
            let (a, rhs) = spd(n);
            let mut ac = a.clone();
            let mut xc = rhs.clone();
            assert!(cholesky_solve(&mut ac, n, &mut xc, 4), "cholesky n={n}");
            let mut ag = a.clone();
            let mut xg = rhs.clone();
            assert!(gauss_jordan(&mut ag, n, &mut xg, 4), "gauss-jordan n={n}");
            assert!(residual(&a, n, &xc, &rhs) < 1e-8, "cholesky n={n}");
            assert!(residual(&a, n, &xg, &rhs) < 1e-8, "gauss-jordan n={n}");
            for (c, g) in xc.iter().zip(&xg) {
                assert!((c - g).abs() < 1e-8, "n={n}: {c} vs {g}");
            }
            // El Gauss–Jordan por bloques da el mismo paso, con uno o con varios hilos.
            for threads in [1usize, 4] {
                let mut ab = a.clone();
                let mut xb = rhs.clone();
                assert!(gauss_jordan_solve(&mut ab, n, &mut xb, threads), "bloques n={n}");
                assert!(residual(&a, n, &xb, &rhs) < 1e-8, "bloques n={n}");
                for (c, g) in xc.iter().zip(&xb) {
                    assert!((c - g).abs() < 1e-8, "bloques n={n}: {c} vs {g}");
                }
            }
        }
    }

    /// ε₁ de la malla con colas, por la fórmula cerrada y por la integral de Kramers–Kronig
    /// hecha con trapecios sobre la misma ε₂.
    #[test]
    fn tails_match_the_principal_value_integral() {
        let anchors = vec![9.0, 10.0, 14.0, 21.0, 30.0, 31.0];
        let a = vec![0.0, 2.0, 0.7, 3.0, 1.5, 0.0];
        let eps2 = |x: f64| eps2_value(&anchors, &a, x, true);
        for om in [3.0, 9.5, 12.0, 17.3, 26.0, 45.0, 90.0] {
            let closed = kk_values(&anchors, &a, &[om], true)[0];
            // P∫ x·ε₂(x)/(x² − ω²) dx = ∫ x·[ε₂(x) − ε₂(ω)]/(x² − ω²) dx + ε₂(ω)·½·ln|(X² − ω²)/ω²|,
            // en una malla logarítmica de 1e-6 a X = 1e6.
            let here = eps2(om);
            let (lo, hi, steps) = (1e-6f64, 1e6f64, 400_000usize);
            let h = (hi / lo).ln() / steps as f64;
            let f = |x: f64| {
                let den = x * x - om * om;
                if den.abs() < 1e-12 * om * om {
                    0.0
                } else {
                    x * (eps2(x) - here) / den * x
                }
            };
            let mut sum = 0.5 * (f(lo) + f(hi));
            for k in 1..steps {
                sum += f(lo * (h * k as f64).exp());
            }
            let numeric = 2.0 / PI * (sum * h + here * 0.5 * ((hi * hi - om * om) / (om * om - lo * lo)).abs().ln());
            assert!((closed - numeric).abs() < 2e-3 * (1.0 + numeric.abs()), "ω = {om}: {closed} vs {numeric}");
        }
    }

    #[test]
    fn tail_kernel_series_joins_the_closed_form() {
        let a = 4000.0;
        for om in [199.0, 200.0, 201.0] {
            let u: f64 = om / a;
            let c = 1.0 / (u * u * u);
            let direct = ((1.0 - c) * (a - om).ln() + (1.0 + c) * (a + om).ln() - 2.0 / (u * u) + 2.0) / PI;
            assert!((tail_high_kernel(om, a) - direct).abs() < 1e-7, "ω = {om}");
        }
    }

    #[test]
    fn gauss_jordan_leaves_the_inverse() {
        let n = 40;
        let (a, rhs) = spd(n);
        let mut inv = a.clone();
        let mut x = rhs.clone();
        assert!(gauss_jordan(&mut inv, n, &mut x, 1));
        for i in 0..n {
            for j in 0..n {
                let s: f64 = (0..n).map(|k| a[i * n + k] * inv[k * n + j]).sum();
                let want = if i == j { 1.0 } else { 0.0 };
                assert!((s - want).abs() < 1e-9, "({i},{j}) {s}");
            }
        }
    }

    #[test]
    fn singular_systems_are_refused() {
        let mut a = vec![1.0, 1.0, 1.0, 1.0];
        let mut b = vec![1.0, 2.0];
        assert!(!cholesky_solve(&mut a, 2, &mut b, 1));
        let mut a = vec![0.0, 0.0, 0.0, 0.0];
        let mut b = vec![1.0, 2.0];
        assert!(!gauss_jordan(&mut a, 2, &mut b, 1));
        let mut a = vec![1.0, 1.0, 1.0, 1.0];
        let mut b = vec![1.0, 2.0];
        assert!(!gauss_jordan_solve(&mut a, 2, &mut b, 1));
    }

    #[test]
    fn strided_mesh_keeps_both_ends_and_nests() {
        let coarse = strided(101, 8);
        let fine = strided(101, 4);
        assert_eq!(coarse.first(), Some(&0));
        assert_eq!(coarse.last(), Some(&100));
        assert!(coarse.iter().all(|i| fine.contains(i)));
        assert_eq!(strided(5, 1), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn prolongation_keeps_the_function() {
        let w: Vec<f64> = (0..40).map(|i| 10.0 + i as f64).collect();
        let cand: Vec<f64> = w.windows(2).map(|p| 0.5 * (p[0] + p[1])).collect();
        let curve = None;
        let cfg = Config {
            mesh: MeshMode::Full,
            solver: Solver::Auto,
            iter_max: 1,
            cancel: None,
            progress_step: None,
            progress_curve: &curve,
            candidates: &cand,
            edge_lo: 9.0,
            edge_hi: 50.0,
            tails: false,
            want_errors: false,
        };
        let coarse = build_mesh(&cfg, &strided(cand.len(), 4), &w);
        let fine = build_mesh(&cfg, &strided(cand.len(), 1), &w);
        let mut p: Vec<f64> = (0..coarse.interior()).map(|j| (j as f64 * 0.7).sin() + 2.0).collect();
        p.push(3.0);
        let q = prolong(&coarse, &p, &fine);
        assert_eq!(q.len(), fine.interior() + 1);
        assert_eq!(*q.last().unwrap(), 3.0);
        for om in [9.5, 12.25, 20.0, 33.3, 48.9] {
            assert!((coarse.eps2_at(om, &p) - fine.eps2_at(om, &q)).abs() < 1e-12, "ω = {om}");
        }
    }

    #[test]
    fn noise_ignores_a_real_peak() {
        let w: Vec<f64> = (0..400).map(|i| i as f64).collect();
        let mut seed = 7u64;
        let y: Vec<f64> = w
            .iter()
            .map(|&x| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let u = ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5;
                1.0 / (1.0 + (x - 200.0) * (x - 200.0) / 9.0) + 0.002 * u
            })
            .collect();
        let s = noise_level(&w, &y);
        assert!(s > 1e-4 && s < 2e-3, "ruido {s}");
    }
}
