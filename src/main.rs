mod collectors;
mod dtos;
mod utils;

use collectors::{
    games_collecting::games_manager::GamesManager,
    languages_collecting::languages_manager::LanguagesManager,
};

use dtos::{program_output_dto::ProgramOutputDto, review::Review, review_register::ReviewRegister};
use utils::io::IO;

use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::io;
use std::time::Instant;

const CANT_LANGUAGES_PER_GAME: usize = 3;
const CANT_REVIEWS_PER_LANGUAGE: usize = 10;
const CANT_TOP_K_GAMES: usize = 3;
const CANT_TOP_K_LANGUAGES: usize = 3;
const PADRON: u32 = 110450;
const ERROR_RAYON_POOL_THREADS: &str = "Error al configurar el pool de hilos de Rayon";

fn main() -> io::Result<()> {
    let (directory_path, num_threads, output_filename) = IO::parse_command_line_args();
    set_rayon_workers(num_threads);
    let iter_reviews = IO::par_iter_into_rewiews(directory_path);

    let start = Instant::now();
    let (games_manager, languages_manager) = par_proccessing(iter_reviews);

    println!(" - Con {:?} workers tardo: {:?}" , num_threads, start.elapsed());

    IO::save_json(
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

fn par_proccessing(
    reviews_iter: impl ParallelIterator<Item = ReviewRegister>,
) -> (GamesManager, LanguagesManager) {
    let (mut games_manager, mut languages_manager) =
        reviews_iter
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

    (games_manager, languages_manager)
}

fn set_rayon_workers(num_workers: usize) {
    ThreadPoolBuilder::new()
        .num_threads(num_workers)
        .build_global()
        .expect(ERROR_RAYON_POOL_THREADS);
}
