use lightmatter::{
    fit::{parse_spectrum, run_fit, FitRequest, Physics},
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
        eprintln!("usage: cargo run --release --example spectrum_bench -- <r|eps1|eps2> <file.dat> [iterations]");
        std::process::exit(2);
    };
    let Some(path) = args.next() else {
        eprintln!("usage: cargo run --release --example spectrum_bench -- <r|eps1|eps2> <file.dat> [iterations]");
        std::process::exit(2);
    };
    let measure = Measure::parse(&measure_arg);
    let iter_max = args.next().and_then(|value| value.parse().ok()).unwrap_or(10);
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
        smoothness: 0.08,
        negativity: 100.0,
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