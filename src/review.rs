use std::cmp::Ordering;

#[derive(Clone)]
pub struct Review {
    /// Review: contiene el contenido de un review y la cantidad de votos que indican que es útil.
    cont: String,
    votes: u32,
}
impl Review {
    /// Método que crea un nuevo Review con el contenido y la cantidad de votos que indican que es útil.
    pub fn new(cont: String, votes: u32) -> Self {
        Review { cont, votes }
    }
}
/// Método que considera un nuevo review en el idioma, actualizando la cuenta de reviews del idioma y considerando si el nuevo review es útil para el top k reviews
impl PartialEq for Review {
    fn eq(&self, other: &Self) -> bool {
        self.votes == other.votes
    }
}

impl Eq for Review {}

impl PartialOrd for Review {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Review {
    fn cmp(&self, other: &Self) -> Ordering {
        self.votes.cmp(&other.votes)
    }
}
