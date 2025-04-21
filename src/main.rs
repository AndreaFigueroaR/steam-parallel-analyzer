use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::fs::File;
use csv::Reader;
use rayon::prelude::*;
use std::cmp::Reverse;


mod language_info;
mod language_manager;
mod game_info;
mod language_top_review;

use crate::game_info::GameInfo;

mod review_register;
use crate::review_register::ReviewInfo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //poría intentar hacer algo para separar el archivo original en varios archivos temporales con diferentes registros
    let file =
        File::open("/home/andy//Escritorio/ProgConcu/data-set-tp1/steam_reviews.csv")?;
    let reader = Reader::from_reader(file);

    let games_map = reader
        .into_deserialize::<ReviewInfo>()
        .par_bridge()
        .fold(
            || HashMap::new(),
            |mut local_map: HashMap<u32, GameInfo>, result| {
                let ReviewInfo { app_id, app_name: game, language, review: cont, votes_helpful: votes, .. } = result.unwrap();
                if local_map.contains_key(&app_id){
                    local_map
                        .get_mut(&app_id)
                        .unwrap()
                        .consider_review(language, cont, votes);
                } else {
                    local_map.insert(app_id, GameInfo::new(game, language, cont, votes));
                }
                local_map
            },
        )
        .reduce(
          || std::collections::HashMap::new(),
          |mut global_acc:HashMap<u32,GameInfo>, local_map: HashMap<u32,GameInfo>| {

            local_map.into_iter()
                .for_each(|( app_id, game_info)| {
                    if global_acc.contains_key(&app_id) {
                        global_acc.get_mut(&app_id).unwrap().join(game_info);
                    } else {
                        global_acc.insert(app_id, game_info);
                    }
                });
            global_acc
        });
        let grand_theft_id : u32 = 271590;
        let tom_clancy_id :u32 = 359550;
        let player_uknown_id:u32= 578080;

        if games_map.contains_key(&tom_clancy_id){
            let game_info = games_map.get(&tom_clancy_id).unwrap();
            println!("El juego {:?} tiene {:?} reviews", game_info.game, game_info.review_count);
        }
        if games_map.contains_key(&grand_theft_id){
            let game_info = games_map.get(&grand_theft_id).unwrap();
            println!("El juego {:?} tiene {:?} reviews", game_info.game, game_info.review_count);
        }
        if games_map.contains_key(&player_uknown_id){
            let game_info = games_map.get(&player_uknown_id).unwrap();
            println!("El juego {:?} tiene {:?} reviews", game_info.game, game_info.review_count);
        }
    //inlusive si falla el resultado esto está bien.


    //OBTENGO SOLO LOS K GAMES CON MÁS REVIEWS: ojo aquí ha de estar fallando.
    //IMPLEMENTACIÓN DE TOP K paralelo:
    let k =3;
    let mut heap_complete_top_games_info = games_map.into_par_iter()
        .map(|(_, v)| v)
        .fold(||BinaryHeap::<Reverse<GameInfo>>::new(),|mut local_acc,game_info|{
            if local_acc.len() < k {
                local_acc.push(Reverse(game_info));
            } else if local_acc.peek().unwrap().0 < game_info {
                local_acc.pop();
                local_acc.push(Reverse(game_info));
            }
            local_acc
        })
        .reduce(||BinaryHeap::<Reverse<GameInfo>>::new(),|mut global_acc,mut local_acc: BinaryHeap<Reverse<GameInfo>>|{ 
            while let Some(Reverse(game_info)) = local_acc.pop() {
                if global_acc.len() < k {
                    global_acc.push(Reverse(game_info));
                } else if global_acc.peek().unwrap().0 <= game_info {
                    global_acc.push(Reverse(game_info));
                } 
            }   
            global_acc
        });

    let mut complete_top_games_info = Vec::with_capacity(heap_complete_top_games_info.len());
    while let Some(Reverse(game_info))= heap_complete_top_games_info.pop(){
        complete_top_games_info.push(game_info);
    }
    complete_top_games_info.reverse();
    
    // for i in 0..k {
    //     println!("El {:?}avo juego con más reviews es: {:?} con {:?} reviews",i, complete_top_games_info[i].game,complete_top_games_info[i].review_count );
    // } 

    // hago la slección de los idiomas de cada juego de forma secuencial
    complete_top_games_info.iter_mut().for_each(|complete_game_info|{
        complete_game_info.par_filter_languages_to_highest_k(k);
    });
    Ok(())
}