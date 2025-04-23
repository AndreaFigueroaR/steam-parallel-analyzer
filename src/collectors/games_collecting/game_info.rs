use crate::collectors::languages_collecting::languages_manager::LanguagesManager;
use crate::dtos::{game_dto::GameDto, review::Review};
use std::{cmp::Ordering, usize};

/// Estructura util para colectar la información de un juego en particular.
/// Además de contener información relativa al juego como tal (id, nombre, cuenta de cuantos reviews hubo de este), se colecta tambipen información de las reviews del mismo segregadas por el lenguage: esto mediante la composicón con un `LanguageManager``
pub struct GameInfo {
    /// ID del juego (único) del que la instancia  guarda y guardará información.
    id: u32,
    /// Nombre del juego del que se está guardando información.
    game: String,
    /// Cuenta de cuantos reviews se han considerado en la presente instancia de información del juego.
    review_count: u32,
    /// Administrador de lenguages que guardará solo `CANT_REVIEWS_PER_LANGUAGE` reviews por cada lnguage registrado en reviews del juego.
    languages: LanguagesManager,
}

impl GameInfo {
    /// Constante de cuantas reviews (con mayor cantidad de votos) por lenguage se enecsitan mantener para la información del juego.
    /// Como solo se necesita la más votada por cada lenguage el valor es 1.
    pub const CANT_REVIEWS_PER_LANGUAGE: usize = 1;

    /// Crea una instancia de un `GameInfo` sabiendo que para un juego sea registrado debe havber sido encontrado mediante una Review
    /// La review del juego debe proporcionar información básica sobre el mismo: el id del mismo, el nombre del juego, el lenguage de la review del juego así como también la información propia al contenido de la review.
    /// El contenido de la review en  determinado lenguage es considerado para mantener el registro de el top review de cada lenguage (relativo a este juego).
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

    /// Getter del id del juego sobre el cual esta instancia guarda información.
    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// Método que considera una review unitaria (debe ser relativa al mismo juego) en la información que se recopila  en la instancia de `GameInfo` actual.
    /// - En caso de tratar de incorporar información de una review juego distinto se lanza un panic.
    /// De brindar información nueva válida del juego: 
    /// - El review afectará la cuenta de reviews del mismo
    /// - Y también los datos segregados por lengauge que se guardan (no necesariamente se almacena, solo de ser parte del top `CANT_REVIEWS_PER_LANGUAGE` del lenguage en el que está el review).
    pub fn consider_review(&mut self, app_id: u32, language: String, review: Review) {
        assert!(self.id == app_id, "Se trató de incorporar una review de un juego distinto al que abarca la instancia de `GameInfo");
        self.review_count += 1;
        self.languages.consider_review(language, review);
    }

    /// Método que junta el contenido del otro `GameInfo` dado en esta instancia.
    /// Después de llamar a este método l:
    /// - La instancia de `GameInfo` recibida como argumento no podrá ser usada pues se toma el ownership del mismo.
    /// - La instancia actual tendrá considerada en la información que retiene los datos de ambas instancias. 
    pub fn join(&mut self, other: GameInfo) {
        assert!(self.id == other.id);
        assert!(self.game == other.game);
        self.review_count += other.review_count;
        self.languages.join(other.languages);
    }

    /// Método que concurrentemente, mediante reduce la información guardada por cada idioma de los reviews del juego para dejar solo la información de los top k lenguages con más reviews o en caso de empate or orden alfabético.
    /// No altera el orden con respecto a otros GameInfos.
    pub fn par_filter_languages_to_highest_k(&mut self, k: usize) {
        self.languages.par_filter_top_k_languages(k);
    }

    pub fn into_serializable(self) -> GameDto {
        GameDto::new(
            self.game,
            self.review_count,
            self.languages.into_serializable(),
        )
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

// Ord: primero por review_count, luego por game
impl Ord for GameInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.review_count.cmp(&other.review_count) {
            Ordering::Equal => self.game.cmp(&other.game),
            other_ordering => other_ordering,
        }
    }
}
