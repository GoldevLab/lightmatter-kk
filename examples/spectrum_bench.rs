use lightmatter::{
    fit::{parse_spectrum, run_fit, Extrapolation, FitRequest, MeshMode, Physics, Solver},
    measure::{Measure, Unit},
};
use std::{
    env,
    error::Error,
    fs,
    io,
    time::Instant,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let Some(measure_arg) = args.next() else {
        eprintln!("usage: cargo run --release --example spectrum_bench -- <r|eps1|eps2> <file.dat> [iterations] [mesh] [solver] [negativity] [dump.tsv] [edges]");
        std::process::exit(2);
    };
    let Some(path) = args.next() else {
        eprintln!("usage: cargo run --release --example spectrum_bench -- <r|eps1|eps2> <file.dat> [iterations] [mesh] [solver] [negativity] [dump.tsv] [edges]");
        std::process::exit(2);
    };
    let measure = Measure::parse(&measure_arg);
    let iter_max = args.next().and_then(|value| value.parse().ok()).unwrap_or(10);
    let mesh = MeshMode::parse(&args.next().unwrap_or_default());
    let solver = Solver::parse(&args.next().unwrap_or_default());
    let negativity = args.next().and_then(|value| value.parse().ok()).unwrap_or(100.0);
    let dump = args.next().filter(|path| path != "-");
    let edges = Extrapolation::parse(&args.next().unwrap_or_default());
    let text = fs::read_to_string(&path)?;
    let (x, y, sigma, errors_given) = parse_spectrum(&text)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let point_count = x.len();
    let started = Instant::now();
    let report = run_fit(&FitRequest {
        x,
        y,
        iter_max,
        log_mesh: true,
        drude: None,
        smoothness: 0.0,
        negativity,
        extended: false,
        sigma,
        errors_given,
        physics: Physics {
            measure,
            unit: Unit::Cm,
            thickness_um: 0.0,
            temperature_k: 300.0,
            eps_inf: None,
        },
        cancel: None,
        progress_step: None,
        progress_curve: None,
        mesh,
        solver,
        edges,
    })
    .map_err(|error| io::Error::other(error))?;
    let rmse = (report
        .model_q
        .iter()
        .zip(&report.data_y)
        .map(|(model, data)| (model - data).powi(2))
        .sum::<f64>()
        / point_count as f64)
        .sqrt();

    let worst = report
        .model_q
        .iter()
        .zip(&report.data_y)
        .zip(&report.data_x)
        .map(|((model, data), x)| ((model - data).abs(), *x))
        .fold((0.0, 0.0), |best, item| if item.0 > best.0 { item } else { best });
    let mut rough = 0.0;
    for trio in report.eps2.windows(3) {
        rough += (trio[0] - 2.0 * trio[1] + trio[2]).abs();
    }
    let negatives = report.eps2.iter().filter(|v| **v < 0.0).count();
    println!(
        "mesh={} solver={} levels={} eps_inf={:.4} max_residual={:.3e}@{:.1} eps2_roughness={:.4e} eps2_negative={negatives} eps2_max={:.3}",
        report.mesh_used,
        report.solver_used,
        report.mesh_levels,
        report.eps_inf.unwrap_or(f64::NAN),
        worst.0,
        worst.1,
        rough / report.eps2.len().max(1) as f64,
        report.eps2.iter().cloned().fold(f64::MIN, f64::max),
    );
    if let Some(path) = dump {
        let mut out = String::from("omega\teps1\teps2\tR\n");
        for i in 0..report.omega.len() {
            out.push_str(&format!("{}\t{}\t{}\t{}\n", report.omega[i], report.eps1[i], report.eps2[i], report.reflectivity[i]));
        }
        fs::write(&path, out)?;
        let mut fit = String::from("omega\tdata\tmodel\n");
        for i in 0..report.data_x.len() {
            fit.push_str(&format!("{}\t{}\t{}\n", report.data_x[i], report.data_y[i], report.model_q[i]));
        }
        fs::write(format!("{path}.fit"), fit)?;
    }
    println!(
        "file={path} measure={} points={point_count} anchors={} mfit={} iterations={} converged={} chi2={:.6e} rmse={:.6e} elapsed={:.2?}",
        measure.key(),
        report.n_anchor,
        report.mfit,
        report.iters_used,
        report.converged,
        report.chisq,
        rmse,
        started.elapsed(),
    );
    for warning in report.warnings {
        println!("warning={warning}");
    }
    Ok(())
}