use crate::dtos::review::Review;
use serde::ser::{Serialize, SerializeStruct, Serializer};

const UNIQUE_LANGUAGE: usize = 1;
const BASE_COUNT_FILEDS_DESERIALIZATION: usize = 2;

const HEADER_LANGUAGE_NAME: &str = "language";
const HEADER_REVIEW_COUNT: &str = "review_count";
const HEADERS_SINGLE_TOP_REVIEW: [&str; 2] = ["top_review", "top_review_votes"];
const HEADER_MULT_TOP_REVIEWS: &str = "top_reviews";

/// Version of LanguageReviewsInfo deserializable
pub struct LanguageDto {
    language: String,
    review_count: u32,
    top_reviews: Vec<Review>,
}

impl LanguageDto {
    pub fn new(language: String, review_count: u32, top_reviews: Vec<Review>) -> Self {
        LanguageDto {
            language,
            review_count,
            top_reviews,
        }
    }
}

impl Serialize for LanguageDto {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        //Calculo de los campos
        let fields_cant = BASE_COUNT_FILEDS_DESERIALIZATION
            + if self.top_reviews.len() == UNIQUE_LANGUAGE {
                2
            } else {
                1
            };
        let mut state = serializer.serialize_struct("LanguageDto", fields_cant)?;

        //Campos siempre presentes
        state.serialize_field(HEADER_LANGUAGE_NAME, &self.language)?;
        state.serialize_field(HEADER_REVIEW_COUNT, &self.review_count)?;

        //FOrmato diferente según la cantidad de top_reviews
        if self.top_reviews.len() == UNIQUE_LANGUAGE {
            let top_review = &self.top_reviews[UNIQUE_LANGUAGE - 1];
            state.serialize_field(HEADERS_SINGLE_TOP_REVIEW[0], &top_review.review)?;
            state.serialize_field(HEADERS_SINGLE_TOP_REVIEW[1], &top_review.votes)?;
        } else {
            state.serialize_field(HEADER_MULT_TOP_REVIEWS, &self.top_reviews)?;
        }

        state.end()
    }
}
