mod collectors;
mod dtos;
mod utils;

use collectors::{
    games_collecting::games_manager::GamesManager,
    languages_collecting::languages_manager::LanguagesManager,
};

use dtos::{program_output_dto::ProgramOutputDto, review::Review, review_register::ReviewRegister};

use csv::Reader;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use serde_json;
use std::env;
use std::fs::{File, read_dir};
use std::io;
use std::io::BufWriter;
use std::time::Instant;

const CANT_LANGUAGES_PER_GAME: usize = 3;
const CANT_REVIEWS_PER_LANGUAGE: usize = 10;

const CANT_TOP_K_GAMES: usize = 3;
const CANT_TOP_K_LANGUAGES: usize = 3;

const PADRON: u32 = 110450;

fn main() -> io::Result<()> {
    let (directory_path, num_threads, output_filename) = arguments_parsing();

    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .expect("Error al configurar el pool de hilos de Rayon");

    let registers_par_iterator = get_par_iter_throgh_rewiews(directory_path);
    //comienza el procesamiento: inicio a tomar el tiempo:
    let start = Instant::now();
    let (mut games_manager, mut languages_manager) =
        registers_par_iterator
            .fold(
                || {
                    (
                        GamesManager::new(),
                        LanguagesManager::new(CANT_REVIEWS_PER_LANGUAGE),
                    )
                },
                |(mut local_games_manager, mut local_language_manager): (
                    GamesManager,
                    LanguagesManager,
                ),
                 register| {
                    let ReviewRegister {
                        app_id,
                        app_name: game,
                        language,
                        review: cont,
                        votes_helpful: votes,
                        ..
                    } = register;
                    let review_cont = Review::new(cont, votes);
                    local_games_manager.consider_review(
                        app_id,
                        game,
                        language.clone(),
                        review_cont.clone(),
                    );
                    local_language_manager.consider_review(language, review_cont);
                    (local_games_manager, local_language_manager)
                },
            )
            .reduce(
                || (GamesManager::new(), LanguagesManager::new(10)),
                |(mut global_games_manager, mut global_language_manager): (
                    GamesManager,
                    LanguagesManager,
                ),
                 (local_games_manager, local_language_manager): (
                    GamesManager,
                    LanguagesManager,
                )| {
                    global_games_manager.join(local_games_manager);
                    global_language_manager.join(local_language_manager);
                    (global_games_manager, global_language_manager)
                },
            );

    games_manager.par_filter_top_k_games(CANT_TOP_K_GAMES);
    games_manager.filter_games_top_k_languages_par(CANT_LANGUAGES_PER_GAME);

    languages_manager.par_filter_top_k_languages(CANT_TOP_K_LANGUAGES);

    //printeo cuanto tiempo me tardé en procesar la data (no cuento cuanto tarde escribirla)
    println!("{:?}", start.elapsed());
    save_json(
        output_filename,
        ProgramOutputDto::new(
            PADRON,
            games_manager.into_serializable(),
            languages_manager.into_serializable(),
        ),
    )
    .unwrap();
    Ok(())
}

fn get_par_iter_throgh_rewiews(
    directory_path: String,
) -> impl ParallelIterator<Item = ReviewRegister> {
    read_dir(directory_path)
        .unwrap()
        .par_bridge()
        .map(|d| d.unwrap().path())
        .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "csv"))
        .flat_map(|path| {
            let file = File::open(path).unwrap();
            let reader = Reader::from_reader(file);
            reader
                .into_deserialize::<ReviewRegister>()
                .par_bridge()
                .filter_map(Result::ok)
        })
}

fn save_json<T: serde::Serialize>(path: String, output: T) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &output).unwrap();
    Ok(())
}

fn arguments_parsing() -> (String, usize, String) {
    let mut args = env::args().skip(1);

    let directory_path = args
        .next()
        .expect("Uso: programa <DIRECTORIO> <NUM_HILOS> <OUTPUT_FILENAME>");

    let num_threads: usize = args
        .next()
        .expect("Uso: programa <DIRECTORIO> <NUM_HILOS> <OUTPUT_FILENAME>")
        .parse()
        .expect("El número de worker threads debe ser un entero positivo");

    let output_filename = args
        .next()
        .expect("Uso: programa <DIRECTORIO> <NUM_HILOS> <OUTPUT_FILENAME>");

    (directory_path, num_threads, output_filename)
}
