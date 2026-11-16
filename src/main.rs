use steam_reviews::charts::ChartsRenderer;
use steam_reviews::processing::process_directory;
use steam_reviews::utils::io::IO;

use rayon::ThreadPoolBuilder;
use std::io;
use std::process::ExitCode;
use std::time::Instant;

const ERROR_RAYON_POOL_THREADS: &str = "Error al configurar el pool de hilos de Rayon";

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
    let args = IO::parse_command_line_args().map_err(io::Error::other)?;
    set_rayon_workers(args.num_threads)?;

    let start = Instant::now();
    let output = process_directory(&args.directory_path)?;
    println!(
        " - Con {} workers tardó: {:?}",
        args.num_threads,
        start.elapsed()
    );

    IO::save_json(&args.output_filename, &output)?;

    if let Some(charts_directory) = args.charts_directory {
        let generated = ChartsRenderer::new(&charts_directory)?.render_metrics(&output)?;
        println!(
            " - Gráficos generados en '{charts_directory}': {}",
            generated.join(", ")
        );
    }

    Ok(())
}

fn set_rayon_workers(num_workers: usize) -> io::Result<()> {
    ThreadPoolBuilder::new()
        .num_threads(num_workers)
        .build_global()
        .map_err(|err| io::Error::other(format!("{ERROR_RAYON_POOL_THREADS}: {err}")))
}
