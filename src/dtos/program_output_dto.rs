use crate::dtos::{game_dto::GameDto, language_dto::LanguageDto};
use serde::{Deserialize, Serialize};
use std::{fs::File, io, io::BufReader};

#[derive(Debug, Serialize, Deserialize)]
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

    /// Lee un JSON de salida previamente generado (por ejemplo, para graficarlo sin reprocesar el dataset).
    pub fn from_json_file(path: &str) -> io::Result<Self> {
        let reader = BufReader::new(File::open(path)?);
        serde_json::from_reader(reader).map_err(io::Error::other)
    }

    pub fn top_games(&self) -> &[GameDto] {
        &self.top_games
    }

    pub fn top_languages(&self) -> &[LanguageDto] {
        &self.top_languages
    }
}
