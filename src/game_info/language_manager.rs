use std::collections::HashMap;

#[derive(Debug)]
struct LanguageTopReview {
    cont: String,
    votes: u64
}

#[derive(Debug)]
struct LanguageReviewInfo{
    //cuenta de los reviews en este idioma
    review_count: u32,
    top_review: LanguageTopReview
}

#[derive(Debug)]
pub struct LanguageManager {
    languages: HashMap<String, LanguageReviewInfo>,
}

impl LanguageManager {
    pub fn new() -> Self {
        LanguageManager {
            languages: HashMap::new(),
        }
    }

    //considera el  review en cualquier idioma (del juego)
    pub fn consider_review(&mut self, language: String, cont: String, votes: u64) {
        if self.languages.contains_key(&language){
            let language_state = self.languages.get_mut(&language).unwrap();
            language_state.review_count += 1;
            if language_state.top_review.votes < votes {
                language_state.top_review = LanguageTopReview{ cont, votes } ;
            }
        }else{
            self.languages.insert(language, LanguageReviewInfo {
                review_count: 1,
                top_review: LanguageTopReview { cont, votes }
            });

        }
        
    }
    pub fn join(&mut self, other: LanguageManager) {
        other.languages.into_iter().for_each(|(language, new_data)| {
            // si este elemento (data del lenguage) ya existe en el mapa, lo actualizo
            if self.languages.contains_key(&language){
                let this_language = self.languages.get_mut(&language).unwrap();
                this_language.review_count += new_data.review_count;
                if this_language.top_review.votes < new_data.top_review.votes {
                    this_language.top_review = new_data.top_review;
                }
            } else {
                // si no existía data del lenguage entonces sinmplemente lo agrego
                self.languages.insert(language, new_data);
            }            
        });

    }
}