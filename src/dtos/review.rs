use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Review {
    pub review: String,
    pub votes: u32,
}

impl Review {
    /// Método que crea un nuevo Review con el contenido y la cantidad de votos que indican que es útil.
    pub fn new(review: String, votes: u32) -> Self {
        Review { review, votes }
    }

    /// Indica si una review (todavía no materializada, con su texto prestado) quedaría por encima de `other`
    /// según el mismo orden que `Ord for Review`. Permite decidir si vale la pena copiar el texto.
    pub fn ranks_above(votes: u32, review: &str, other: &Review) -> bool {
        (votes, review) > (other.votes, other.review.as_str())
    }
}

impl PartialEq for Review {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Review {}

impl PartialOrd for Review {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Orden primero por votos y, ante empate, por contenido. El desempate hace que el resultado sea
/// determinista sin importar el orden en que los workers procesen los registros.
impl Ord for Review {
    fn cmp(&self, other: &Self) -> Ordering {
        self.votes
            .cmp(&other.votes)
            .then_with(|| self.review.cmp(&other.review))
    }
}
