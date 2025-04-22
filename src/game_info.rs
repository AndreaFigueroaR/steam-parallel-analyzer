use crate::languages_manager::LanguagesManager;
use crate::review::Review;
use std::{cmp::Ordering, usize};


#[derive(Debug)]
pub struct GameInfo {
    pub id: u32,
    pub game : String,
    pub review_count: u32,//cuantos reviews hemos visto de este juego
    pub languages: LanguagesManager
}

impl GameInfo {
    //crea su languageManager con la configuración para que solo mantenga el top review
    pub fn new(id :u32, game: String, language: String, review: Review ) -> Self {
        //TODO: tener como constante este uno hardcodeado
        let mut languages = LanguagesManager::new(1);
        languages.consider_review(language, review);
        GameInfo { id, game, review_count: 1, languages }
    }

    /// Método para considerar el contenido de un review unitario nuevo en el `GameInfo` actual.
    /// Para lo cual se actualiza la cuenta de reviews de este juego así como también se incorporan los datos en el 'languageManager' interno
    /// El langugage manager mantendrá actualizado cual es el review su correspondiente lenguage con la mayor cantidad de votos como útil.
    pub fn consider_review(&mut self, language: String, review: Review) {
        self.review_count += 1;
        self.languages.consider_review(language, review);
    }

    /// Método de un GameInfo que se encarga de juntar dos GamesInfo de un mismo juego en dicha instancia. 
    /// La cantidad de reviews de este juego se actualiza como tambien se 
    pub fn join(&mut self, other: GameInfo) {
        assert!(self.id == other.id);
        assert!(self.game == other.game);
        self.review_count += other.review_count;
        self.languages.join(other.languages);
    }
    
    /// metodo que concurrentemente reduce la información guardada por cada idioma de los reviews del juego para dejar solo la información de los top k lenguages con más reviews o en caso de empate or orden alfabético.
    /// No altera el orden con respecto a otros GameInfos.
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
