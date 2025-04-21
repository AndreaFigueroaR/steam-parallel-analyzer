use crate::language_info::LanguageReviewsInfo;
use crate::language_top_review::LanguageTopReview;
use rayon::prelude::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct LanguageManager {
    pub languages: HashMap<String, LanguageReviewsInfo>,
}

impl LanguageManager {
    pub fn new() -> Self {
        LanguageManager {
            languages: HashMap::new(),
        }
    }

    // Método del manager de lenguages que considera el  review en cualquier idioma (del juego) actualizando los idiomas registrados de reviews del juego así como (de existir) actualizando la información preexistente relativa a dicho idioma
    pub fn consider_review(&mut self, language: String, cont: String, votes: u64) {
        if self.languages.contains_key(&language){
            let language_state = self.languages.get_mut(&language).unwrap();
            language_state.consider_new_review(cont, votes);
        }else{
            self.languages.insert(language.clone(), LanguageReviewsInfo {
                name: language,
                review_count: 1,
                top_review: LanguageTopReview { cont, votes }
            });

        }
        
    }
    
    // metodo para fusionar dos Languages Manangers
    pub fn join(&mut self, other: LanguageManager) {
        other.languages.into_iter().for_each(|(language, new_data)| {
            if self.languages.contains_key(&language){
                let this_language: &mut LanguageReviewsInfo = self.languages.get_mut(&language).unwrap();
                this_language.join(new_data);
            } else {
                self.languages.insert(language, new_data);
            }            
        });

    }

    /// Método que concurrentemente se encarga de dejar los lenguages guardados internamente solo con los top k lenguages
    /// Para lo cual se tiene en consideración el orden establecido por los `LanguageReviewsInfo` (primero por cantidad de reviews y luego por orden alfabético) 
    pub fn par_filter_to_highest_k(&mut self, k: usize){
        if self.languages.len() <= k {
            return;
        }
        let mut items: Vec<(String, LanguageReviewsInfo)> = self.languages.drain().collect();
        items.par_sort_by(|(_, a), (_, b)| b.cmp(a));
        items.truncate(k);
        self.languages = items.into_iter().collect();
    }
}