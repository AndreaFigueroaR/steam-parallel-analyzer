use crate::collectors::languages_collecting::language_reviews_info::LanguageReviewsInfo;
use crate::dtos::{language_dto::LanguageDto, review::Review};
use crate::utils::top_k::TopK;

use rayon::prelude::*;
use std::collections::HashMap;
/// Estructura que se maneja las `top_reviews_retained` reviews más importantes (más votadas) consideradas en esta instancia, segregandolas por lenguage. 
/// Proporciona  ina interfaz segura mediante la cual se puede considerar nuevas reviews dentro de la información relativa al lenguage en la que está dicha review. 
/// Al considerar una review esta puede o no terminar siendo literalmente almacenada (dependiendo de si es de las más votadas) pero siempre influirá en la cuenta de reviews de su lenguage.
pub struct LanguagesManager {
    /// Lenguages que se han considerado en esta instancia junto a sus top `top_reviews_retained` reviews (las de más votos).
    languages: HashMap<String, LanguageReviewsInfo>,
    /// Cantidad de reviews de cada idioma que serán literalmente guardadas (contenido y su cantidad de calificaciones como util: `votes`)
    top_reviews_retained: usize, 
}

impl LanguagesManager {
    /// Crea un nuevo `LanguagesManager` recibiendo como argumento la cantidad de reviews que guardará por lenguage como máximo.
    pub fn new(reviews_to_retain: usize) -> Self {
        LanguagesManager {
            languages: HashMap::new(),
            top_reviews_retained: reviews_to_retain,
        }
    }

    /// Método para considerar un review unitario, en cualquier idioma, actualizando el estado del idioma si ya fue registrado (considerarlo para el top k) o en caso de no haberse registrado previamente se crea la  información del idioma basándose en esta primera review.
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
    /// El método hace implementa de forma paralela el algoritmo de Top K (n log k) haciendo uso de la utilidad `TopK` que se encuentra en el módulo `utils/top_k.rs`
    pub fn par_filter_top_k_languages(&mut self, k: usize) {
        if self.languages.len() <= k {
            return;
        }
        let items : Vec<(String,LanguageReviewsInfo)> = self.languages.drain().collect();
        self.languages = items
            .into_par_iter()
            .map(|(_, language_info)| language_info)
            .fold(
                || TopK::<LanguageReviewsInfo>::new(k),
                |mut local_acc, language_info| {
                    local_acc.consider_include(language_info);
                    local_acc
                },
            )
            .reduce(
                || TopK::<LanguageReviewsInfo>::new(k),
                |mut global_acc, local_acc| {
                    local_acc.into_unsorted().into_iter().for_each(|language_info| {
                        global_acc.consider_include(language_info);
                    });
                    global_acc
                },
            )
            .into_unsorted()
            .into_iter()
            .map(|language_info| (language_info.get_name(), language_info))
            .collect();
        
    }

    /// Método que toma la propiedad de la instancia de `LanguagesManager` (posteriormente no se podrá usar) y devuelve su versión como data transfer object.
    /// El vector de `LanguageDto` devuelto en escencia contienen la información que anteriormente tenía el `LanguagesManager` en un formato serializable. 
    pub fn into_serializable(self) -> Vec<LanguageDto> {
        self.languages
            .into_iter()
            .map(|(_, language_info)| language_info.into_serializable())
            .collect()
    }
}
