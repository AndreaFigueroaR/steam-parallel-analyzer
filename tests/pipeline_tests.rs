use rayon::ThreadPoolBuilder;
use steam_reviews::dtos::program_output_dto::ProgramOutputDto;
use steam_reviews::processing::{process_directory, process_directory_with_chunk_size};

const SAMPLE_DIRECTORY: &str = "data";
/// Bloques chicos (en bytes) para que las ~440 reviews de la muestra se repartan entre decenas de
/// bloques y workers. Varios registros son más largos que el bloque, lo que ejercita ese caso borde.
const SMALL_CHUNK: usize = 2048;

fn process_with_threads(threads: usize) -> String {
    let pool = ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();
    let output = pool
        .install(|| process_directory_with_chunk_size(SAMPLE_DIRECTORY, SMALL_CHUNK))
        .expect("no se pudo procesar la muestra");
    serde_json::to_string(&output).unwrap()
}

/// El resultado debe ser idéntico (byte a byte) sin importar la cantidad de workers:
/// la fusión de acumuladores parciales es asociativa y todos los empates se desempatan.
#[test]
fn output_is_deterministic_across_thread_counts() {
    let reference = process_with_threads(1);
    let pool = ThreadPoolBuilder::new().num_threads(1).build().unwrap();
    let default_batches = pool
        .install(|| process_directory(SAMPLE_DIRECTORY))
        .unwrap();
    assert_eq!(reference, serde_json::to_string(&default_batches).unwrap());
    for threads in [2, 4, 8, 16] {
        assert_eq!(
            reference,
            process_with_threads(threads),
            "resultado distinto con {threads} workers"
        );
    }
}

#[test]
fn sample_output_has_expected_shape() {
    let pool = ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    let output = pool
        .install(|| process_directory(SAMPLE_DIRECTORY))
        .unwrap();

    let games = output.top_games();
    assert!(!games.is_empty() && games.len() <= 3);
    assert!(
        games
            .windows(2)
            .all(|w| w[0].review_count() >= w[1].review_count())
    );
    for game in games {
        assert!(game.languages().len() <= 3);
        assert!(game.languages().iter().all(|l| l.top_reviews().len() == 1));
    }

    let languages = output.top_languages();
    assert!(languages.len() <= 3);
    for language in languages {
        let votes: Vec<u32> = language.top_reviews().iter().map(|r| r.votes).collect();
        assert!(votes.len() <= 10);
        assert!(votes.windows(2).all(|w| w[0] >= w[1]));
    }
}

#[test]
fn missing_directory_is_an_error_not_a_panic() {
    assert!(process_directory("directorio/inexistente").is_err());
}

/// El JSON de referencia del dataset completo se puede leer y volver a serializar sin pérdida.
#[test]
fn expected_output_roundtrip() {
    let output = ProgramOutputDto::from_json_file("expected_output.json").unwrap();
    let original: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string("expected_output.json").unwrap()).unwrap();
    let reserialized = serde_json::to_value(&output).unwrap();
    assert_eq!(original["top_games"], reserialized["top_games"]);
    assert_eq!(original["top_languages"], reserialized["top_languages"]);
}

/// El corte en bloques no pierde ni parte registros: la cantidad total de reviews contabilizadas
/// coincide con la de una lectura secuencial con el crate `csv`, para distintos tamaños de bloque.
#[test]
fn chunking_preserves_every_record() {
    let mut reader = csv::Reader::from_path("data/steam_reviews_sample.csv").unwrap();
    let expected = reader
        .deserialize::<serde_json::Value>()
        .filter(Result::is_ok)
        .count() as u32;

    let pool = ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    for chunk_size in [1, 100, 2048, 64 * 1024, 16 * 1024 * 1024] {
        let output = pool
            .install(|| process_directory_with_chunk_size(SAMPLE_DIRECTORY, chunk_size))
            .unwrap();
        let counted: u32 = output.top_games().iter().map(|g| g.review_count()).sum();
        assert_eq!(expected, counted, "bloques de {chunk_size} bytes");
    }
}
