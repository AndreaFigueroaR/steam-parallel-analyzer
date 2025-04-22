mod review_register;
mod review;
mod language_reviews_info;
mod languages_manager;
mod game_info;
mod games_manager;
mod utils;


use std::fs::File;
use csv::Reader;
use rayon::prelude::*;

use crate::review_register::ReviewInfo;
use crate::review::Review;
use crate::languages_manager::LanguagesManager;
use crate::games_manager::GamesManager;


const CANT_LANGUAGES_PER_GAME: usize = 3;
const CANT_REVIEWS_PER_LANGUAGE:usize = 10;

const CANT_TOP_K_GAMES: usize = 3;
const CANT_TOP_K_LANGUAGES:usize = 3;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file =
        File::open("/home/andy//Escritorio/ProgConcu/data-set-tp1/steam_reviews.csv")?;
    let reader = Reader::from_reader(file);

    let (mut games_manager,mut languages_manager) = reader
        .into_deserialize::<ReviewInfo>()
        .par_bridge()
        .fold(
            || (GamesManager::new(),LanguagesManager::new(CANT_REVIEWS_PER_LANGUAGE)),
            |(mut local_games_manager, mut local_language_manager): (GamesManager,LanguagesManager), result_reading| {
                let ReviewInfo { app_id, app_name: game, language, review: cont, votes_helpful: votes, .. } = result_reading.unwrap();
                let review_cont = Review{ cont, votes };
                local_games_manager.consider_review(app_id, game, language.clone(), review_cont.clone());
                local_language_manager.consider_review(language, review_cont);
                (local_games_manager,local_language_manager)
            },
        ).reduce(|| (GamesManager::new(), LanguagesManager::new(10)),
          |(mut global_games_manager,mut global_language_manager):(GamesManager, LanguagesManager), (local_games_manager,local_language_manager): (GamesManager,LanguagesManager)| {
            global_games_manager.join(local_games_manager);
            global_language_manager.join(local_language_manager);
            (global_games_manager,global_language_manager)
        });


    games_manager.par_filter_top_k_games(CANT_TOP_K_GAMES);
    games_manager.filter_games_top_k_languages_par(CANT_LANGUAGES_PER_GAME);

    languages_manager.par_filter_top_k_languages(CANT_TOP_K_LANGUAGES);
    
    println!("{:?}",languages_manager);

    //TODO implementar deserializador 
    Ok(())
}