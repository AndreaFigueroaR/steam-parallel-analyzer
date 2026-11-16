use crate::dtos::{language_dto::LanguageDto, review::Review};
use crate::utils::top_k::TopK;

use std::cmp::Ordering;

/// LanguageReviewsInfo: contiene, respeto a un idioma, el nombre, la cantidad de reviews que se registraron en este idioma y sus top reviews en este idioma (contenido y cantidad de votos que indican que es útil)
pub struct LanguageReviewsInfo {
    language: String,
    review_count: u32,
    top_reviews: TopK<Review>,
}

impl LanguageReviewsInfo {
    /// Método que crea un nuevo LanguageReviewsInfo con el nombre del idioma, a partir de la información del primer review en este idioma registrado.
    pub fn new(reviews_to_retain: usize, language: &str, review: &str, votes: u32) -> Self {
        let mut info = LanguageReviewsInfo {
            language: language.to_owned(),
            review_count: 0,
            top_reviews: TopK::new(reviews_to_retain),
        };
        info.consider_new_review(review, votes);
        info
    }

    pub fn get_name(&self) -> &str {
        &self.language
    }

    /// Método que considera un nuevo review en el idioma, actualizando la cuenta de reviews del idioma y considerando si el nuevo review es útil para el top k reviews del lenguage.
    /// El texto de la review solo se copia si efectivamente entra al top k (caso poco frecuente una vez que el top se estabiliza).
    pub fn consider_new_review(&mut self, review: &str, votes: u32) {
        self.review_count += 1;
        if self
            .top_reviews
            .admits(|smallest| Review::ranks_above(votes, review, smallest))
        {
            self.top_reviews
                .consider_include(Review::new(review.to_owned(), votes));
        }
    }

    /// Método que junta la información de otro LanguageReviewsInfo (otra info del mismo lenguage) en esta instancia.
    /// Actualizando la cuenta de reviews del lenguage y actualizando cuáles son los reviews más útiles en este lenguage
    pub fn join(&mut self, other: LanguageReviewsInfo) {
        assert!(self.language == other.language);
        self.review_count += other.review_count;
        for value in other.top_reviews.into_unsorted() {
            self.top_reviews.consider_include(value);
        }
    }

    /// Convierte la información en su DTO con las reviews ordenadas de más a menos votada.
    /// `single_top_review` indica si se serializa en el formato de una única top review.
    pub fn into_serializable(self, single_top_review: bool) -> LanguageDto {
        LanguageDto::new(
            self.language,
            self.review_count,
            self.top_reviews.into_sorted_desc(),
            single_top_review,
        )
    }
}

impl PartialEq for LanguageReviewsInfo {
    /// Implementación personalizada que considera para la comparación de LanguageReviewsInfo solo la cantidad de reviews en dicho lenguage y en caso de "empate" el nombre del lenguage
    fn eq(&self, other: &Self) -> bool {
        self.review_count == other.review_count && self.language == other.language
    }
}

/// Se delega la igualdad al comportamiento de PartialEq (considera cantidad de reviews y nombre del lenguage)
impl Eq for LanguageReviewsInfo {}

impl PartialOrd for LanguageReviewsInfo {
    /// Implementación de comparación parcial entre dos `LanguageReviewsInfo` que delega en Ord::cmp el resultado
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LanguageReviewsInfo {
    /// Implementación que establece el el orden de los `LanguageReviewsInfo` primero por `review_count` y si ambos tienen la misma cantidad de reseñas, desempata usando el nombre del idioma.
    fn cmp(&self, other: &Self) -> Ordering {
        self.review_count
            .cmp(&other.review_count)
            .then_with(|| self.language.cmp(&other.language))
    }
}
