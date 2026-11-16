use steam_reviews::collectors::{
    games_collecting::games_manager::GamesManager,
    languages_collecting::languages_manager::LanguagesManager,
};
use steam_reviews::dtos::review_register::ReviewRegister;

/// (app_id, app_name, language, review, votes). Incluye a propósito un id con dos nombres distintos
/// y empates de votos, que son los casos en los que el resultado podría depender del reparto.
const ROWS: [(u32, &str, &str, &str, u32); 12] = [
    (1, "Alpha", "english", "great", 5),
    (1, "Alpha", "spanish", "genial", 5),
    (2, "Beta", "english", "meh", 1),
    (1, "Alpha (old name)", "english", "fine", 5),
    (3, "Gamma", "russian", "ok", 0),
    (2, "Beta", "english", "bad", 9),
    (1, "Alpha", "english", "wow", 2),
    (3, "Gamma", "english", "nice", 9),
    (2, "Beta", "spanish", "malo", 3),
    (1, "Alpha", "russian", "hmm", 9),
    (3, "Gamma", "spanish", "bien", 5),
    (1, "Alpha", "spanish", "otra", 5),
];

fn register(row: &(u32, &'static str, &'static str, &'static str, u32)) -> ReviewRegister<'static> {
    ReviewRegister {
        app_id: row.0,
        app_name: row.1,
        language: row.2,
        review: row.3,
        votes_helpful: row.4,
    }
}

/// Simula un worker: acumula su partición en managers locales (fase fold).
fn fold(
    rows: &[(u32, &'static str, &'static str, &'static str, u32)],
) -> (GamesManager, LanguagesManager) {
    let mut games = GamesManager::new();
    let mut languages = LanguagesManager::new(2);
    for row in rows {
        let register = register(row);
        games.consider_review(&register);
        languages.consider_review(&register);
    }
    (games, languages)
}

/// Fusiona los acumuladores parciales en el orden dado (fase reduce) y serializa el resultado.
fn reduce_and_serialize(parts: Vec<(GamesManager, LanguagesManager)>) -> String {
    let (mut games, mut languages) = (GamesManager::new(), LanguagesManager::new(2));
    for (part_games, part_languages) in parts {
        games.join(part_games);
        languages.join(part_languages);
    }
    games.par_filter_top_k_games(2);
    games.filter_games_top_k_languages(2);
    languages.par_filter_top_k_languages(2);
    serde_json::to_string(&(games.into_serializable(), languages.into_serializable())).unwrap()
}

#[test]
fn join_is_independent_of_partitioning_and_merge_order() {
    let sequential = reduce_and_serialize(vec![fold(&ROWS)]);

    for chunk_size in 1..ROWS.len() {
        let mut parts: Vec<_> = ROWS.chunks(chunk_size).map(fold).collect();
        assert_eq!(
            sequential,
            reduce_and_serialize(parts),
            "chunks de {chunk_size}"
        );

        parts = ROWS.chunks(chunk_size).map(fold).collect();
        parts.reverse();
        assert_eq!(
            sequential,
            reduce_and_serialize(parts),
            "chunks de {chunk_size} invertidos"
        );
    }
}

#[test]
fn game_name_is_chosen_deterministically() {
    let output = reduce_and_serialize(vec![fold(&ROWS[3..]), fold(&ROWS[..3])]);
    assert!(output.contains("\"game\":\"Alpha (old name)\""));
    assert!(!output.contains("\"game\":\"Alpha\""));
}
