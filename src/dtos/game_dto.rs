use crate::dtos::language_dto::LanguageDto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GameDto {
    game: String,
    review_count: u32,
    languages: Vec<LanguageDto>,
}

impl GameDto {
    pub fn new(game: String, review_count: u32, languages: Vec<LanguageDto>) -> Self {
        GameDto {
            game,
            review_count,
            languages,
        }
    }

    pub fn game(&self) -> &str {
        &self.game
    }

    pub fn review_count(&self) -> u32 {
        self.review_count
    }

    /// Idiomas con más reviews del juego, de mayor a menor.
    pub fn languages(&self) -> &[LanguageDto] {
        &self.languages
    }
}
