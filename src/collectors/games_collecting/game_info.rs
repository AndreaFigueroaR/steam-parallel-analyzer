use crate::collectors::languages_collecting::languages_manager::LanguagesManager;
use crate::dtos::{game_dto::GameDto, review_register::ReviewRegister};
use std::cmp::Ordering;

/// Estructura util para colectar la información de un juego en particular de forma eficiente.
/// Además de contener información relativa al juego como tal (id, nombre, cuenta de cuantos reviews hubo de este), se colecta también información de las reviews del mismo segregadas por el lenguage: esto mediante la composición con un `LanguagesManager`
pub struct GameInfo {
    /// ID del juego (único) del que la instancia guarda y guardará información.
    id: u32,
    /// Nombre del juego del que se está guardando información.
    game: String,
    /// Cuenta de cuantos reviews se han considerado en la presente instancia de información del juego.
    review_count: u32,
    /// Administrador de lenguages que guardará solo `CANT_REVIEWS_PER_LANGUAGE` reviews por cada lenguage registrado en reviews del juego.
    languages: LanguagesManager,
}

impl GameInfo {
    /// Constante de cuantas reviews (con mayor cantidad de votos) por lenguage se necesitan mantener para la información del juego.
    /// Como solo se necesita la más votada por cada lenguage el valor es 1.
    pub const CANT_REVIEWS_PER_LANGUAGE: usize = 1;

    /// Crea una instancia de un `GameInfo` sabiendo que para que un juego sea registrado debe haber sido encontrado mediante una review.
    /// La review debe proporcionar información básica sobre el juego (id, nombre, idioma) y el contenido propio de la review,
    /// que es considerado para mantener el registro del top review de cada lenguage (relativo a este juego).
    pub fn new(register: &ReviewRegister) -> Self {
        let mut languages = LanguagesManager::new(GameInfo::CANT_REVIEWS_PER_LANGUAGE);
        languages.consider_review(register);
        GameInfo {
            id: register.app_id,
            game: register.app_name.to_owned(),
            review_count: 1,
            languages,
        }
    }

    /// Getter del id del juego sobre el cual esta instancia guarda información.
    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// Método que considera una review unitaria (debe ser relativa al mismo juego) en la información que se recopila en la instancia de `GameInfo` actual.
    /// - En caso de tratar de incorporar información de una review de un juego distinto se lanza un panic.
    ///
    /// De brindar información nueva válida del juego:
    /// - El review afectará la cuenta de reviews del mismo
    /// - Y también los datos segregados por lenguage que se guardan (no necesariamente se almacena, solo de ser parte del top `CANT_REVIEWS_PER_LANGUAGE` del lenguage en el que está el review).
    pub fn consider_review(&mut self, register: &ReviewRegister) {
        assert!(
            self.id == register.app_id,
            "Se trató de incorporar una review de un juego distinto al que abarca la instancia de `GameInfo`"
        );
        self.consider_name(register.app_name);
        self.review_count += 1;
        self.languages.consider_review(register);
    }

    /// El juego se identifica por su id. Si el dataset trae nombres distintos para un mismo id, se
    /// conserva el mayor alfabéticamente: una regla asociativa y conmutativa, de modo que el nombre final
    /// no dependa de qué worker procesó cada registro ni del orden en que se fusionan los resultados.
    /// En el caso habitual (nombre idéntico) cuesta una comparación de pocos bytes y no asigna memoria.
    fn consider_name(&mut self, name: &str) {
        if name > self.game.as_str() {
            self.game = name.to_owned();
        }
    }

    /// Método que junta el contenido del otro `GameInfo` dado en esta instancia.
    /// Después de llamar a este método:
    /// - La instancia de `GameInfo` recibida como argumento no podrá ser usada pues se toma el ownership del mismo.
    /// - La instancia actual tendrá considerada en la información que retiene los datos de ambas instancias.
    /// - Si ambas instancias tienen nombres distintos para el mismo id se aplica la regla de `consider_name`.
    pub fn join(&mut self, other: GameInfo) {
        assert!(self.id == other.id);
        if other.game > self.game {
            self.game = other.game;
        }
        self.review_count += other.review_count;
        self.languages.join(other.languages);
    }

    /// Método que altera los reviews propiamente guardados (segregados por lenguage) del juego para solo retener los `k` lenguages con más reviews.
    /// El método no afecta el orden respecto a otras instancias de `GameInfo`
    pub fn par_filter_languages_to_highest_k(&mut self, k: usize) {
        self.languages.par_filter_top_k_languages(k);
    }

    /// Método que toma la propiedad de la instancia de `GameInfo` (posteriormente no se podrá usar) y devuelve su versión como data transfer object.
    /// La estructura `GameDto` devuelta en esencia contiene la misma información que anteriormente tenía el `GameInfo` en un formato serializable.
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

// Establezco orden que considere primero el campo `review_count` luego el campo `game` (nombre del juego)
impl Ord for GameInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.review_count
            .cmp(&other.review_count)
            .then_with(|| self.game.cmp(&other.game))
    }
}
