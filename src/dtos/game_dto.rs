use crate::dtos::language_dto::LanguageDto;
use serde::Serialize;

#[derive(Serialize)]
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
}
