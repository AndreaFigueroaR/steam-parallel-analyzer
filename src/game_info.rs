use crate::language_manager::LanguageManager;
use std::{cmp::Ordering, usize};


#[derive(Debug)]
pub struct GameInfo {
    pub game : String,
    pub review_count: u32,//cuantos reviews hemos visto de este juego
    languages: LanguageManager
}

impl GameInfo {

    pub fn new(game: String, language: String, cont: String, votes:u64 ) -> Self {
        let mut languages = LanguageManager::new();
        languages.consider_review(language, cont, votes);
        GameInfo { game,review_count: 1, languages }
    }

    /// Método para considerar el contenido de un review unitario nuevo en el `GameInfo` actual.
    /// Para lo cual se actualiza la cuenta de reviews de este juego así como también se incorporan los datos en el 'languageManager' interno
    /// El langugage manager mantendrá actualizado cual es el review su correspondiente lenguage con la mayor cantidad de votos como útil.
    pub fn consider_review(&mut self, language: String, cont: String, votes:u64) {
        self.review_count += 1;
        self.languages.consider_review(language, cont, votes);
    }

    /// Método de un GameInfo que se encarga de juntar dos GamesInfo de un mismo juego en dicha instancia. 
    /// La cantidad de reviews de este juego se actualiza como tambien se 
    pub fn join(&mut self, other: GameInfo) {
        self.review_count += other.review_count;
        self.languages.join(other.languages);
    }
    
    /// metodo que concurrentemente reduce la información guardada por cada idioma de los reviews del juego para dejar solo la información de los top k lenguages con más reviews o en caso de empate or orden alfabético.
    pub fn par_filter_languages_to_highest_k(&mut self, k: usize){
        self.languages.par_filter_to_highest_k(k);
    }
}


impl PartialEq for GameInfo {
    fn eq(&self, other: &Self) -> bool {
        self.review_count == other.review_count
            && self.game == other.game
    }
}

impl Eq for GameInfo {}

impl PartialOrd for GameInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// 4) Ord: primero por review_count, luego por game
impl Ord for GameInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.review_count.cmp(&other.review_count) {
            Ordering::Equal => self.game.cmp(&other.game),
            other_ordering => other_ordering,
        }
    }
}
