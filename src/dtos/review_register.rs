use serde::Deserialize;
use serde::de::Deserializer;

// structure only used to obtain useful data from the registers
// when reading tyhe csv data file
#[derive(Debug, Deserialize)]
pub struct ReviewRegister {
    pub app_id: u32,
    pub app_name: String,
    pub language: String,
    pub review: String,
    #[serde(deserialize_with = "de_votes_helpful")]
    pub votes_helpful: u32,
}

/// Función de deserialización personalizada para `votes_helpful` en vista de que hay al menos un registro que desborda el rango de u32
fn de_votes_helpful<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s.parse::<u32>() {
        Ok(v) => Ok(v),
        Err(_) => Ok(0),
    }
}
