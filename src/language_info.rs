use crate::language_top_review::LanguageTopReview;

use std::cmp::Ordering;

/// LanguageReviewsInfo: contiene, respeto a un idioma, el nombre, la cantidad de reviews que se registraronen este idioma y el top review en este idioma (contenido y cantidad de votos que indican que es útil)
#[derive(Debug)]
pub struct LanguageReviewsInfo{
    pub name: String,
    pub review_count: u32,//cuenta de los reviews en este idioma
    pub top_review: LanguageTopReview
}
impl LanguageReviewsInfo {
    /// Método que junta la información de otro LanguageReviewsInfo (otra info del mismo lenguage) en esta instana.
    /// Actualizando la cuenta de reviews del lenguage y actualizando cuál es el review más útil en este lenguage
    pub fn join(&mut self, other: LanguageReviewsInfo){
        self.review_count += other.review_count;
        if self.top_review.votes < other.top_review.votes {
            self.top_review = other.top_review;
        }
    }

    
    pub fn consider_new_review(&mut self, cont: String, votes:u64){
        self.review_count += 1;
        if self.top_review.votes < votes {
            self.top_review = LanguageTopReview{ cont, votes } ;
        }
    }
}


impl PartialEq for LanguageReviewsInfo {
    /// Implementación personalizada que considera para la comparación de LanguageReviewsInfo solo la cantidad de reviews en dicho lenguage y en caso de "empate" el nombre del lenguage
    fn eq(&self, other: &Self) -> bool {
        self.review_count == other.review_count
            && self.name == other.name
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
        match self.review_count.cmp(&other.review_count) {
            Ordering::Equal => self.name.cmp(&other.name),
            other_ordering => other_ordering,
        }
    }
}