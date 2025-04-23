use crate::dtos::{language_dto::LanguageDto,game_dto::GameDto};
use serde::Serialize;

#[derive(Serialize)]
pub struct ProgramOutputDto {
    pub padron: u32,
    pub top_games: Vec<GameDto>,
    pub top_languages: Vec<LanguageDto>
}