use crate::language_reviews_info::LanguageReviewsInfo;
use crate::review::Review;
use rayon::prelude::*;

use std::collections::HashMap;

pub struct LanguagesManager {
    languages: HashMap<String, LanguageReviewsInfo>,
    top_reviews_retained: usize, //Cantidad de top reviews de cada idioma a retener (para games solo es 1, para lenguages son los 10)
}

impl LanguagesManager {
    pub fn new(reviews_to_retain: usize) -> Self {
        LanguagesManager {
            languages: HashMap::new(),
            top_reviews_retained: reviews_to_retain,
        }
    }

    // Método del manager de lenguages que considera el  review en cualquier idioma actualizando los idiomas registrados de reviews del juego así como (de existir) actualizando la información preexistente relativa a dicho idioma
    pub fn consider_review(&mut self, language: String, review: Review) {
        if self.languages.contains_key(&language) {
            self.languages
                .get_mut(&language)
                .unwrap()
                .consider_new_review(review);
        } else {
            self.languages.insert(
                language.clone(),
                LanguageReviewsInfo::new(self.top_reviews_retained, language, review),
            );
        }
    }

    // metodo para fusionar dos Languages Manangers
    pub fn join(&mut self, other: LanguagesManager) {
        other
            .languages
            .into_iter()
            .for_each(|(language, new_data)| {
                if self.languages.contains_key(&language) {
                    self.languages.get_mut(&language).unwrap().join(new_data);
                } else {
                    self.languages.insert(language, new_data);
                }
            });
    }

    /// Método que concurrentemente se encarga de dejar los lenguages guardados internamente solo con los top k lenguages
    /// Para lo cual se tiene en consideración el orden establecido por los `LanguageReviewsInfo` (primero por cantidad de reviews y luego por orden alfabético)
    pub fn par_filter_top_k_languages(&mut self, k: usize) {
        if self.languages.len() <= k {
            return;
        }
        let mut items: Vec<(String, LanguageReviewsInfo)> = self.languages.drain().collect();
        items.par_sort_by(|(_, language_info_a), (_, language_info_b)| {
            language_info_b.cmp(language_info_a)
        });
        items.truncate(k);
        self.languages = items.into_iter().collect();
    }
}
