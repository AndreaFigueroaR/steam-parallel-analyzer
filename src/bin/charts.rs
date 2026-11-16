//! Genera los gráficos de métricas a partir de un JSON de salida ya existente, sin reprocesar el dataset.
//!
//! Uso: charts <OUTPUT_JSON> <DIRECTORIO_GRAFICOS>

use std::env;
use std::process::ExitCode;
use steam_reviews::charts::ChartsRenderer;
use steam_reviews::dtos::program_output_dto::ProgramOutputDto;

const USAGE: &str = "Uso: charts <OUTPUT_JSON> <DIRECTORIO_GRAFICOS>";

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let (Some(json_path), Some(charts_directory)) = (args.next(), args.next()) else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    let result = ProgramOutputDto::from_json_file(&json_path)
        .and_then(|output| ChartsRenderer::new(&charts_directory)?.render_metrics(&output));
    match result {
        Ok(generated) => {
            println!(
                " - Gráficos generados en '{charts_directory}': {}",
                generated.join(", ")
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
