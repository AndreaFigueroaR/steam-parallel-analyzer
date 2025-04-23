use crate::collectors::languages_collecting::languages_manager::LanguagesManager;
use crate::dtos::{review::Review, game_dto::GameDto};
use std::{cmp::Ordering, usize};

pub struct GameInfo {
    id: u32,
    game: String,
    review_count: u32, //cuantos reviews hemos visto de este juego
    languages: LanguagesManager,
}


impl GameInfo {
    pub const CANT_REVIEWS_PER_LANGUAGE: usize = 1;

    //crea su languageManager con la configuración para que solo mantenga el top review
    pub fn new(id: u32, game: String, language: String, review: Review) -> Self {
        let mut languages = LanguagesManager::new(GameInfo::CANT_REVIEWS_PER_LANGUAGE);
        languages.consider_review(language, review);
        GameInfo {
            id,
            game,
            review_count: 1,
            languages,
        }
    }
    pub fn get_id(&self) -> u32 {
        self.id
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
    pub fn par_filter_languages_to_highest_k(&mut self, k: usize) {
        self.languages.par_filter_top_k_languages(k);
    }

    pub fn into_serializable(self) -> GameDto {
        GameDto::new(self.game, self.review_count, self.languages.into_serializable())
    }
}

impl PartialEq for GameInfo {
    fn eq(&self, other: &Self) -> bool {
        self.review_count == other.review_count && self.game == other.game
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
