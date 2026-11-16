use crate::collectors::{
    games_collecting::games_manager::GamesManager,
    languages_collecting::languages_manager::LanguagesManager,
};
use crate::dtos::program_output_dto::ProgramOutputDto;
use crate::utils::io::{CHUNK_SIZE, IO, ReviewsBatch};

use rayon::prelude::*;
use std::io;

pub const CANT_LANGUAGES_PER_GAME: usize = 3;
pub const CANT_REVIEWS_PER_LANGUAGE: usize = 10;
pub const CANT_TOP_K_GAMES: usize = 3;
pub const CANT_TOP_K_LANGUAGES: usize = 3;
pub const PADRON: u32 = 110450;

/// Procesa todos los CSV del directorio dado usando el pool de Rayon activo (el global o el de un
/// `ThreadPool::install`) y devuelve el resultado listo para serializar.
pub fn process_directory(directory_path: &str) -> io::Result<ProgramOutputDto> {
    process_directory_with_chunk_size(directory_path, CHUNK_SIZE)
}

/// Igual que `process_directory` pero con un tamaño de bloque explícito, en bytes (útil para tests:
/// bloques chicos fuerzan a que una muestra pequeña se reparta entre muchos workers).
pub fn process_directory_with_chunk_size(
    directory_path: &str,
    chunk_size: usize,
) -> io::Result<ProgramOutputDto> {
    let batches = IO::par_iter_into_batches(directory_path, chunk_size)?;
    let (games_manager, languages_manager) = par_processing(batches);
    Ok(ProgramOutputDto::new(
        PADRON,
        games_manager.into_serializable(),
        languages_manager.into_serializable(),
    ))
}

/// Construye el par de acumuladores vacíos que usa cada worker (fold) y la identidad del reduce.
fn new_accumulators() -> (GamesManager, LanguagesManager) {
    (
        GamesManager::new(),
        LanguagesManager::new(CANT_REVIEWS_PER_LANGUAGE),
    )
}

/// Pipeline fork-join:
/// - **fork / fold**: cada worker de Rayon toma lotes de registros (work-stealing) y los acumula en
///   su propio par de managers, sin locks ni estado compartido.
/// - **join / reduce**: los acumuladores parciales se fusionan de a pares en forma de árbol.
/// - **filtrado**: sobre el resultado global se calculan los top k (también en paralelo).
pub fn par_processing(
    batches: impl ParallelIterator<Item = ReviewsBatch>,
) -> (GamesManager, LanguagesManager) {
    let (mut games_manager, mut languages_manager) = batches
        .fold(
            new_accumulators,
            |(mut local_games_manager, mut local_languages_manager), batch| {
                batch.for_each_register(|register| {
                    local_games_manager.consider_review(&register);
                    local_languages_manager.consider_review(&register);
                });
                (local_games_manager, local_languages_manager)
            },
        )
        .reduce(
            new_accumulators,
            |(mut global_games_manager, mut global_languages_manager),
             (local_games_manager, local_languages_manager)| {
                global_games_manager.join(local_games_manager);
                global_languages_manager.join(local_languages_manager);
                (global_games_manager, global_languages_manager)
            },
        );

    games_manager.par_filter_top_k_games(CANT_TOP_K_GAMES);
    games_manager.filter_games_top_k_languages(CANT_LANGUAGES_PER_GAME);
    languages_manager.par_filter_top_k_languages(CANT_TOP_K_LANGUAGES);

    (games_manager, languages_manager)
}
