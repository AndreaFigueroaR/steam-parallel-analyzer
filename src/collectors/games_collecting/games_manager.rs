use crate::collectors::games_collecting::game_info::*;
use crate::dtos::{game_dto::GameDto, review::Review};
use crate::utils::top_k::TopK;

use rayon::prelude::*;
use std::collections::HashMap;

pub struct GamesManager {
    games: HashMap<u32, GameInfo>,
}

impl GamesManager {
    pub fn new() -> Self {
        GamesManager {
            games: HashMap::new(),
        }
    }

    pub fn consider_review(&mut self, app_id: u32, game: String, language: String, review: Review) {
        if self.games.contains_key(&app_id) {
            self.games
                .get_mut(&app_id)
                .unwrap()
                .consider_review(language, review);
        } else {
            self.games
                .insert(app_id, GameInfo::new(app_id, game, language, review));
        }
    }

    pub fn join(&mut self, other: GamesManager) {
        other.games.into_iter().for_each(|(app_id, game_info)| {
            if self.games.contains_key(&app_id) {
                self.games.get_mut(&app_id).unwrap().join(game_info);
            } else {
                self.games.insert(app_id, game_info);
            }
        });
    }

    // Teniendo en consideración que la cantidad de juegos es mucho mayor a la cantidad de lenguages-> se decide usar el algoritmo de top k para que el manager de juegos se quede solo con los k más reviewd games.
    // Implementación concurrente
    pub fn par_filter_top_k_games(&mut self, k: usize) {
        //OPCION 1
        let items: Vec<(u32, GameInfo)> = self.games.drain().collect();
        //creo un TopK con el tamaño k y con los valores de las claves
        self.games = items
            .into_par_iter()
            .map(|(_, game_info)| game_info)
            .fold(
                || TopK::<GameInfo>::new(k),
                |mut local_acc, game_info| {
                    local_acc.consider_include(game_info);
                    local_acc
                },
            )
            .reduce(
                || TopK::<GameInfo>::new(k),
                |mut global_acc, local_acc: TopK<GameInfo>| {
                    local_acc.into_unsorted().into_iter().for_each(|game_info| {
                        global_acc.consider_include(game_info);
                    });
                    global_acc
                },
            )
            .into_unsorted()
            .into_iter()
            .map(|game_info| (game_info.get_id(), game_info))
            .collect();
        // como que 78 segundos con esta opcion

        // // OPCION 2
        // if self.games.len() <= k {
        //     return;
        // }
        // let mut items: Vec<(u32, GameInfo)> = self.games.drain().collect();
        // items.par_sort_by(|(_, game_info_a), (_, game_info_b)| {
        //     game_info_b.cmp(game_info_a)
        // });
        // items.truncate(k);
        // self.games = items.into_iter().collect();
        // // // mejora en tiempo: 72 segundos con un hilo
        // // 87.20894438s con dos hilos
    }

    /// Método que filtra cada juego para que solo mantenga los k lenguages con más reviews. De forma que por cada juego se lleve a cabo secuencialmente mientras que por cada lenguage de cada juego se lleva a cabo secuencialmente
    pub fn filter_games_top_k_languages_par(&mut self, k: usize) {
        self.games.iter_mut().for_each(|(_, game_info)| {
            game_info.par_filter_languages_to_highest_k(k);
        });
    }

    pub fn into_serializable(self) -> Vec<GameDto> {
        self.games
            .into_iter()
            .map(|(_, game_info)| game_info.into_serializable())
            .collect()
    }
    
}
