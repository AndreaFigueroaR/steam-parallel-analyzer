use crate::collectors::games_collecting::game_info::*;
use crate::dtos::{game_dto::GameDto, review_register::ReviewRegister};
use crate::utils::top_k::TopK;

use rayon::prelude::*;
use std::collections::{HashMap, hash_map::Entry};

/// Estructura que maneja la información de todos los juegos considerados en esta instancia, indexados por su id.
/// Proporciona una interfaz segura para considerar nuevas reviews de cualquier juego, fusionar instancias parciales
/// (generadas por distintos workers) y quedarse con los juegos con más reviews.
pub struct GamesManager {
    games: HashMap<u32, GameInfo>,
}

impl Default for GamesManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GamesManager {
    pub fn new() -> Self {
        GamesManager {
            games: HashMap::new(),
        }
    }

    /// Método para considerar un review unitario, de cualquier juego, actualizando el estado guardado del juego de ya haber sido registrado o, en caso de no haberse registrado previamente, creando la información del juego basándose en esta primera review.
    /// Usa la API `Entry` para resolver ambos casos con una única búsqueda en el HashMap.
    pub fn consider_review(&mut self, register: &ReviewRegister) {
        self.games
            .entry(register.app_id)
            .and_modify(|game_info| game_info.consider_review(register))
            .or_insert_with(|| GameInfo::new(register));
    }

    /// Método que fusiona la instancia dada de `GamesManager` con la instancia actual.
    /// - La instancia de `GamesManager` recibida como argumento no podrá ser usada pues se toma el ownership del mismo.
    /// - La instancia actual tendrá considerada en la información que retiene los datos de ambas instancias.
    /// - En caso de que haya juegos repetidos en ambas instancias se hace un merge de la información de ambos juegos.
    pub fn join(&mut self, other: GamesManager) {
        for (app_id, game_info) in other.games {
            match self.games.entry(app_id) {
                Entry::Occupied(mut entry) => entry.get_mut().join(game_info),
                Entry::Vacant(entry) => {
                    entry.insert(game_info);
                }
            }
        }
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
                    local_acc
                        .into_unsorted()
                        .into_iter()
                        .for_each(|game_info| global_acc.consider_include(game_info));
                    global_acc
                },
            )
            .into_unsorted()
            .into_iter()
            .map(|game_info| (game_info.get_id(), game_info))
            .collect();
    }

    /// Método que filtra cada juego para que solo mantenga los k lenguages con más reviews.
    /// Se recorre secuencialmente (tras el top k de juegos quedan pocos), mientras que el filtrado de lenguages de cada juego es paralelo.
    pub fn filter_games_top_k_languages(&mut self, k: usize) {
        self.games
            .values_mut()
            .for_each(|game_info| game_info.par_filter_languages_to_highest_k(k));
    }

    /// Método que toma la propiedad de la instancia de `GamesManager` (posteriormente no se podrá usar) y devuelve su versión como data transfer object.
    /// Los juegos se devuelven ordenados de mayor a menor cantidad de reviews.
    pub fn into_serializable(self) -> Vec<GameDto> {
        let mut games: Vec<GameInfo> = self.games.into_values().collect();
        games.sort_unstable_by(|a, b| b.cmp(a));
        games.into_iter().map(GameInfo::into_serializable).collect()
    }
}
