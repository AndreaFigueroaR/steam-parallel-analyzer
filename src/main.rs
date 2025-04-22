mod review_register;
mod review;
mod language_reviews_info;
mod language_manager;
mod game_info;
mod utils;


use std::collections::HashMap;
use std::fs::File;
use csv::Reader;
use rayon::prelude::*;

use crate::review_register::ReviewInfo;
use crate::review::Review;
use crate::game_info::GameInfo;
use crate::utils::TopK;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file =
        File::open("/home/andy//Escritorio/ProgConcu/data-set-tp1/steam_reviews.csv")?;
    let reader = Reader::from_reader(file);

    let games_map = reader
        .into_deserialize::<ReviewInfo>()
        .par_bridge()
        .fold(
            || HashMap::new(),
            |mut local_game_map: HashMap<u32, GameInfo>, result| {
                let ReviewInfo { app_id, app_name: game, language, review: cont, votes_helpful: votes, .. } = result.unwrap();
                
                if local_game_map.contains_key(&app_id){
                    local_game_map.get_mut(&app_id).unwrap()
                            .consider_review(language, Review{cont, votes});
                } else {
                    local_game_map.insert(app_id, GameInfo::new(game, language, Review{cont, votes}));
                }

                local_game_map
            },
        )
        .reduce(
          || std::collections::HashMap::new(),
          |mut global_games_acc:HashMap<u32,GameInfo>, local_map: HashMap<u32,GameInfo>| {

            local_map.into_iter()
                .for_each(|( app_id, game_info)| {
                    if global_games_acc.contains_key(&app_id) {
                        global_games_acc.get_mut(&app_id).unwrap().join(game_info);
                    } else {
                        global_games_acc.insert(app_id, game_info);
                    }
                });
            global_games_acc
        });

    //OBTENGO SOLO LOS K GAMES CON MÁS REVIEWS.
    let k =3;
    let complete_top_games_info = games_map.into_par_iter()
        .map(|(_, game_info)| game_info)
        .fold(||TopK::<GameInfo>::new(k),|mut local_acc,game_info|{
            local_acc.consider_include(game_info);
            local_acc
        })
        .reduce(||TopK::<GameInfo>::new(k),|mut global_acc,local_acc: TopK<GameInfo>|{ 
            //el metodo into_unsorted vacia el local acc
            local_acc.into_unsorted().into_iter().for_each(|game_info| {
                global_acc.consider_include(game_info);
            });
            global_acc
        });

    let resumed_top_games_info: Vec<GameInfo> = complete_top_games_info.into_unsorted()
            .into_iter()
            .map(|mut game_info| {
                game_info.par_filter_languages_to_highest_k(k);
                game_info

            }).collect();
    
    for game_info in resumed_top_games_info{
        println!("game: {:?}", game_info.game);
        println!("review_count: {:?}", game_info.review_count);
        println!("languages: {:?}", game_info.languages);
    }

    Ok(())
}