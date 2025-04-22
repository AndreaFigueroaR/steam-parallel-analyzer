use crate::review::Review;
use crate::utils::TopK;
use std::cmp::Ordering;

/// LanguageReviewsInfo: contiene, respeto a un idioma, el nombre, la cantidad de reviews que se registraronen este idioma y el top review en este idioma (contenido y cantidad de votos que indican que es útil)
pub struct LanguageReviewsInfo {
    name: String,
    review_count: u32, //cuenta de los reviews en este idioma
    top_reviews: TopK<Review>,
}
//tambien podría tener un top k binaryHeap
impl LanguageReviewsInfo {
    /// Método que crea un nuevo LanguageReviewsInfo con el nombre del idioma, a partir de la información del primer review en este idioma registrado.
    /// Para lo cual teiene en cuenta la cantidad de reviews y el top review de
    pub fn new(reviews_to_retain: usize, language: String, review: Review) -> Self {
        let mut top_reviews: TopK<Review> = TopK::new(reviews_to_retain);
        top_reviews.consider_include(review);
        LanguageReviewsInfo {
            name: language,
            review_count: 1,
            top_reviews,
        }
    }
    /// Método que considera un nuevo review en el idioma, actualizando la cuenta de reviews del idioma y considerando si el nuevo review es útil para el top k reviews del lenguage
    pub fn consider_new_review(&mut self, review: Review) {
        self.review_count += 1;
        self.top_reviews.consider_include(review);
    }

    /// Método que junta la información de otro LanguageReviewsInfo (otra info del mismo lenguage) en esta instancia.
    /// Actualizando la cuenta de reviews del lenguage y actualizando cuál es el review más útil en este lenguage
    pub fn join(&mut self, other: LanguageReviewsInfo) {
        assert!(self.name == other.name);
        self.review_count += other.review_count;
        //podría cambiarlo por una implementación de Ord de los Reviews que solo tenga en cuenta cual es la más útil
        //debería hacer un merge de los review counts
        //necesito tratar de insertar todos los elementos del otro LanguageReviewsInfo, no me imprta en qué orden los saco
        for value in other.top_reviews.into_unsorted() {
            self.top_reviews.consider_include(value);
        }
    }
}

impl PartialEq for LanguageReviewsInfo {
    /// Implementación personalizada que considera para la comparación de LanguageReviewsInfo solo la cantidad de reviews en dicho lenguage y en caso de "empate" el nombre del lenguage
    fn eq(&self, other: &Self) -> bool {
        self.review_count == other.review_count && self.name == other.name
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
