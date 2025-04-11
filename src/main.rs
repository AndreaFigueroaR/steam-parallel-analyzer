use std::collections::HashMap;
use std::fs::File;
use csv::Reader;
use rayon::prelude::*;

mod review_register;
use crate::review_register::ReviewInfo;

mod game_info;
use crate::game_info::GameInfo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //poría intentar hacer algo para separar el archivo original en varios archivos temporales con diferentes registros
    let file =
        File::open("/home/andy/Escritorio/ProgConcu/small-data-set-tp1/steam_reviews_small.csv")?;
    let mut reader = Reader::from_reader(file);

    let games_map = reader
    .into_deserialize::<ReviewInfo>()
        .par_bridge()
        .fold(
            || HashMap::new(),
            |mut local_map: HashMap<u32, GameInfo>, result| {

                let ReviewInfo { app_id, app_name: game, language, review: cont, votes_helpful: votes, .. } = result.unwrap();

                if local_map.contains_key(&app_id){
                    local_map.get_mut(&app_id).unwrap().consider_review(language, cont, votes);
                } else {
                    local_map.insert(app_id, GameInfo::new(game, language, cont, votes));
                }
                local_map
            },
        )
        .reduce(
          || std::collections::HashMap::new(),
          |mut acc:HashMap<u32,GameInfo>, local_map: HashMap<u32,GameInfo>| {
            for ( key, value) in local_map {
                if acc.contains_key(&key) {
                    acc.get_mut(&key).unwrap().join(value);
                } else {
                    acc.insert(key, value);
                }
            }
            acc
        });
        //hasta aqui solo conseguíun hash con todos los juegos y sus topreviews de todos los lenguages.
        //lo que quiero ahora es que por cada par key: app_id, Value: GameInfo quedarme solo con los k mayores juegos (en nuestro caso no entendí si eran 3 o 10)

    Ok(())
}