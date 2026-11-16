use crate::dtos::review::Review;
use serde::Deserialize;
use serde::ser::{Serialize, SerializeStruct, Serializer};

const BASE_COUNT_FILEDS_SERIALIZATION: usize = 2;

const HEADER_LANGUAGE_NAME: &str = "language";
const HEADER_REVIEW_COUNT: &str = "review_count";
const HEADERS_SINGLE_TOP_REVIEW: [&str; 2] = ["top_review", "top_review_votes"];
const HEADER_MULT_TOP_REVIEWS: &str = "top_reviews";

/// Versión serializable de `LanguageReviewsInfo`.
/// Admite dos formatos de salida:
/// - una única top review (`top_review` + `top_review_votes`), usado en los idiomas de cada juego;
/// - una lista de top reviews (`top_reviews`), usado en el ranking global de idiomas.
///
/// El formato lo decide quien construye el DTO (según cuántas reviews se retienen por idioma) y no
/// la cantidad de reviews encontradas, para que la forma del JSON no dependa de los datos.
#[derive(Debug, Deserialize)]
#[serde(from = "LanguageDtoRaw")]
pub struct LanguageDto {
    language: String,
    review_count: u32,
    top_reviews: Vec<Review>,
    single_top_review: bool,
}

impl LanguageDto {
    pub fn new(
        language: String,
        review_count: u32,
        top_reviews: Vec<Review>,
        single_top_review: bool,
    ) -> Self {
        LanguageDto {
            language,
            review_count,
            top_reviews,
            single_top_review,
        }
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn review_count(&self) -> u32 {
        self.review_count
    }

    /// Top reviews del idioma, de más a menos votada.
    pub fn top_reviews(&self) -> &[Review] {
        &self.top_reviews
    }
}

impl Serialize for LanguageDto {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let fields_cant = BASE_COUNT_FILEDS_SERIALIZATION
            + if self.single_top_review {
                HEADERS_SINGLE_TOP_REVIEW.len()
            } else {
                1
            };
        let mut state = serializer.serialize_struct("LanguageDto", fields_cant)?;

        //Campos siempre presentes
        state.serialize_field(HEADER_LANGUAGE_NAME, &self.language)?;
        state.serialize_field(HEADER_REVIEW_COUNT, &self.review_count)?;

        //Formato diferente según se retenga una única top review o varias
        if self.single_top_review {
            let top_review = self.top_reviews.first();
            state.serialize_field(
                HEADERS_SINGLE_TOP_REVIEW[0],
                &top_review.map(|r| r.review.as_str()),
            )?;
            state.serialize_field(HEADERS_SINGLE_TOP_REVIEW[1], &top_review.map(|r| r.votes))?;
        } else {
            state.serialize_field(HEADER_MULT_TOP_REVIEWS, &self.top_reviews)?;
        }

        state.end()
    }
}

/// Representación intermedia que acepta cualquiera de los dos formatos al leer un JSON de salida.
#[derive(Deserialize)]
struct LanguageDtoRaw {
    language: String,
    review_count: u32,
    top_review: Option<String>,
    top_review_votes: Option<u32>,
    top_reviews: Option<Vec<Review>>,
}

impl From<LanguageDtoRaw> for LanguageDto {
    fn from(raw: LanguageDtoRaw) -> Self {
        match raw.top_reviews {
            Some(top_reviews) => {
                LanguageDto::new(raw.language, raw.review_count, top_reviews, false)
            }
            None => {
                let top_reviews = raw
                    .top_review
                    .map(|review| Review::new(review, raw.top_review_votes.unwrap_or(0)))
                    .into_iter()
                    .collect();
                LanguageDto::new(raw.language, raw.review_count, top_reviews, true)
            }
        }
    }
}
