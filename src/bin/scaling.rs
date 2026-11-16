//! Benchmark de escalabilidad: procesa el mismo directorio con distintas cantidades de workers,
//! imprime una tabla de tiempos/speedup y genera los gráficos correspondientes.
//!
//! Uso: scaling <DIRECTORIO> <MAX_HILOS> <DIRECTORIO_GRAFICOS> [REPETICIONES]

use rayon::ThreadPoolBuilder;
use std::env;
use std::io;
use std::process::ExitCode;
use std::time::Instant;
use steam_reviews::charts::{ChartsRenderer, ScalingSample};
use steam_reviews::processing::process_directory;

const USAGE: &str = "Uso: scaling <DIRECTORIO> <MAX_HILOS> <DIRECTORIO_GRAFICOS> [REPETICIONES]";
const DEFAULT_REPETITIONS: usize = 3;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let usage = || io::Error::other(USAGE);
    let directory = args.next().ok_or_else(usage)?;
    let max_threads: usize = parse_positive(args.next().ok_or_else(usage)?)?;
    let charts_directory = args.next().ok_or_else(usage)?;
    let repetitions = match args.next() {
        Some(value) => parse_positive(value)?,
        None => DEFAULT_REPETITIONS,
    };

    let mut samples = Vec::new();
    println!("{:>8} | {:>10} | {:>8}", "workers", "tiempo", "speedup");
    for threads in thread_counts(max_threads) {
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(io::Error::other)?;
        // Se toma el mejor de varias corridas para reducir el ruido (caché de disco, scheduler).
        let mut best = f64::INFINITY;
        for _ in 0..repetitions {
            let start = Instant::now();
            pool.install(|| process_directory(&directory))?;
            best = best.min(start.elapsed().as_secs_f64());
        }
        let base = samples.first().map_or(best, |s: &ScalingSample| s.seconds);
        println!("{threads:>8} | {best:>9.3}s | {:>7.2}x", base / best);
        samples.push(ScalingSample {
            threads,
            seconds: best,
        });
    }

    let generated = ChartsRenderer::new(&charts_directory)?.render_scaling(&samples)?;
    println!(
        " - Gráficos generados en '{charts_directory}': {}",
        generated.join(", ")
    );
    Ok(())
}

/// 1, 2, 4, 8, ... hasta `max` (incluyendo siempre `max`).
fn thread_counts(max: usize) -> Vec<usize> {
    let mut counts: Vec<usize> = std::iter::successors(Some(1usize), |n| n.checked_mul(2))
        .take_while(|&n| n < max)
        .collect();
    counts.push(max);
    counts
}

fn parse_positive(value: String) -> io::Result<usize> {
    value
        .parse::<usize>()
        .ok()
        .filter(|&n| n > 0)
        .ok_or_else(|| io::Error::other(format!("'{value}' no es un entero positivo")))
}
