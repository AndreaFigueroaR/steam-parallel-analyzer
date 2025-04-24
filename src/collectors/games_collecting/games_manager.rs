use crate::collectors::games_collecting::game_info::*;
use crate::dtos::{game_dto::GameDto, review::Review};
use crate::utils::top_k::TopK;

use rayon::prelude::*;
use std::collections::HashMap;

/// Estructura que se maneja las `top_reviews_retained` reviews más importantes (más votadas) consideradas en esta instancia, segregandolas por lenguage.
/// Proporciona  ina interfaz segura mediante la cual se puede considerar nuevas reviews dentro de la información relativa al lenguage en la que está dicha review.
/// Al considerar una review esta puede o no terminar siendo literalmente almacenada (dependiendo de si es de las más votadas) pero siempre influirá en la cuenta de reviews de su lenguage.
pub struct GamesManager {
    games: HashMap<u32, GameInfo>,
}

impl GamesManager {
    pub fn new() -> Self {
        GamesManager {
            games: HashMap::new(),
        }
    }

    /// Método para considerar un review unitario, de cualquier juego, actualizando el estado guardado del juegode ya haber sido registrado o en caso de no haberse registrado previamente se crea la  información del juego basándose en esta primera review.
    pub fn consider_review(&mut self, app_id: u32, game: String, language: String, review: Review) {
        if self.games.contains_key(&app_id) {
            self.games
                .get_mut(&app_id)
                .unwrap()
                .consider_review(app_id, language, review);
        } else {
            self.games
                .insert(app_id, GameInfo::new(app_id, game, language, review));
        }
    }

    /// Método que fusiona la instancia dada de `GamesManager` con la instancia actual.
    /// - La instancia de `GamesManager` recibida como argumento no podrá ser usada pues se toma el ownership del mismo.
    /// - La instancia actual tendrá considerada en la información que retiene los datos de ambas instancias.
    /// - En caso de que haya juegos repetidos en ambas instancias se hace un merge de la información de ambos juegos.
    pub fn join(&mut self, other: GamesManager) {
        other.games.into_iter().for_each(|(app_id, game_info)| {
            if self.games.contains_key(&app_id) {
                self.games.get_mut(&app_id).unwrap().join(game_info);
            } else {
                self.games.insert(app_id, game_info);
            }
        });
    }

    /// Método que modifica la instancia actual de `GamesManager` para que solo mantenga los `k` juegos con más reviews.
    /// El filtrado a los k juegos con más reviews se hace de forma paralela mediante una implementación concurrente del algoritmo top k.
    pub fn par_filter_top_k_games(&mut self, k: usize) {
        if self.games.len() <= k {
            return;
        }
        let items: Vec<(u32, GameInfo)> = self.games.drain().collect();
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
    }

    /// Método que filtra cada juego para que solo mantenga los k lenguages con más reviews. De forma que por cada juego se lleve a cabo secuencialmente mientras que por cada lenguage de cada juego se lleva a cabo secuencialmente
    pub fn filter_games_top_k_languages_par(&mut self, k: usize) {
        self.games.iter_mut().for_each(|(_, game_info)| {
            game_info.par_filter_languages_to_highest_k(k);
        });
    }

    /// Método que toma la propiedad de la instancia de `GamesManager` (posteriormente no se podrá usar) y devuelve su versión como data transfer object.
    /// La estructura `Vec<GameDto>` devuelta en escencia contiene la misma información que anteriormente tenía el `GamesManager` en un formato serializable.
    pub fn into_serializable(self) -> Vec<GameDto> {
        self.games
            .into_iter()
            .map(|(_, game_info)| game_info.into_serializable())
            .collect()
    }
}
