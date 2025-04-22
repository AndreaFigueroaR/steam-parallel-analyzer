mod review_register;
mod review;
mod language_reviews_info;
mod languages_manager;
mod game_info;
mod games_manager;
mod utils;


use std::fs::File;
use std::io;
use std::env;
use std::fs::read_dir;
use csv::Reader;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

use crate::review_register::ReviewInfo;
use crate::review::Review;
use crate::languages_manager::LanguagesManager;
use crate::games_manager::GamesManager;


const CANT_LANGUAGES_PER_GAME: usize = 3;
const CANT_REVIEWS_PER_LANGUAGE:usize = 10;

const CANT_TOP_K_GAMES: usize = 3;
const CANT_TOP_K_LANGUAGES:usize = 3;


fn main() ->  io::Result<()>{
    let (directory_path, num_threads) = arguments_parsing();
    
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .expect("Error al configurar el pool de hilos de Rayon");

    let (mut games_manager,mut languages_manager) = 
        read_dir(directory_path).unwrap().par_bridge()
        .map(|d| d.unwrap().path())
        .filter(|path| {
            path.is_file() && path.extension().map_or(false, |ext| ext == "csv")
        }).flat_map(|path| {
            let file = File::open(path).unwrap();
            let reader = Reader::from_reader(file);
            reader.into_deserialize::<ReviewInfo>().par_bridge()
        }).fold( || (GamesManager::new(), LanguagesManager::new(CANT_REVIEWS_PER_LANGUAGE)),
            |(mut local_games_manager, mut local_language_manager): (GamesManager,LanguagesManager), result_reading| {
                let ReviewInfo { app_id, app_name: game, language, review: cont, votes_helpful: votes, .. } = result_reading.unwrap();
                let review_cont = Review::new(cont, votes);
                local_games_manager.consider_review(app_id, game, language.clone(), review_cont.clone());
                local_language_manager.consider_review(language, review_cont);
                (local_games_manager,local_language_manager)
        }).reduce(|| (GamesManager::new(), LanguagesManager::new(10)),
          |(mut global_games_manager,mut global_language_manager):(GamesManager, LanguagesManager), (local_games_manager,local_language_manager): (GamesManager,LanguagesManager)| {
            global_games_manager.join(local_games_manager);
            global_language_manager.join(local_language_manager);
            (global_games_manager,global_language_manager)
        });


    games_manager.par_filter_top_k_games(CANT_TOP_K_GAMES);
    games_manager.filter_games_top_k_languages_par(CANT_LANGUAGES_PER_GAME);

    languages_manager.par_filter_top_k_languages(CANT_TOP_K_LANGUAGES);

    // println!("{:?}",languages_manager);

    //TODO implementar deserializador 
    Ok(())
}

fn arguments_parsing()->(String,usize){
    let mut args = env::args().skip(1);
    
    let directory_path = args
        .next()
        .expect("Uso: programa <DIRECTORIO> <NUM_HILOS>");
    
    let num_threads: usize = args
        .next()
        .expect("Debe indicar también el número de hilos")
        .parse()
        .expect("El número de hilos debe ser un entero positivo");
    (directory_path, num_threads)
}


