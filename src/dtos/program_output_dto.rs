use crate::dtos::{game_dto::GameDto, language_dto::LanguageDto};
use serde::Serialize;

#[derive(Serialize)]
pub struct ProgramOutputDto {
    padron: u32,
    top_games: Vec<GameDto>,
    top_languages: Vec<LanguageDto>,
}

impl ProgramOutputDto {
    pub fn new(padron: u32, top_games: Vec<GameDto>, top_languages: Vec<LanguageDto>) -> Self {
        ProgramOutputDto {
            padron,
            top_games,
            top_languages,
        }
    }
}
