use serde::Deserialize;
use serde::de::Deserializer;

/// Estructura usada solo para obtener los datos útiles de cada registro del csv.
/// Los campos de texto se toman prestados del registro crudo (zero-copy): solo se copian a memoria
/// propia cuando efectivamente se almacenan (juego/idioma nuevo o review que entra a un top k).
#[derive(Debug, Deserialize)]
pub struct ReviewRegister<'a> {
    pub app_id: u32,
    pub app_name: &'a str,
    pub language: &'a str,
    pub review: &'a str,
    #[serde(deserialize_with = "de_votes_helpful")]
    pub votes_helpful: u32,
}

/// Función de deserialización personalizada para `votes_helpful` en vista de que hay al menos un registro que desborda el rango de u32
fn de_votes_helpful<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(s.parse::<u32>().unwrap_or(0))
}
