pub mod language_manager;

use language_manager::LanguageManager;

#[derive(Debug)]
pub struct GameInfo {
    game : String,
    review_count: u32,//cuantos reviews hemos visto de este juego
    languages: LanguageManager
}

impl GameInfo {

    pub fn new(game: String, language: String, cont: String, votes:u64 ) -> Self {
        let mut languages = LanguageManager::new();
        languages.consider_review(language, cont, votes);
        GameInfo { game,review_count: 1, languages }
    }

    // considera en la data del juego la información de un nuevo review
    pub fn consider_review(&mut self, language: String, cont: String, votes:u64) {
        //para considerar un nuevo review de un determinado juego tengo que aumentar el contador del game Info
        self.review_count += 1;
        //también tengo que considerar en sus lenguajes este review
        self.languages.consider_review(language, cont, votes);
    }
    pub fn join(&mut self, other: GameInfo) {
        //aumento el contador de reviews
        self.review_count += other.review_count;
        //el merge de GameInfo se hace cuando tengo dos GameInfo del mismo juego.
        //tendría que también juntar dos language_managers... podría hacerlo manualmente pero prefiero pedirselo 
        self.languages.join(other.languages);
    }
}
