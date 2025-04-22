use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Review {
    /// Review: contiene el contenido de un review y la cantidad de votos que indican que es útil. 
    pub cont: String,
    pub votes: u64
}

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