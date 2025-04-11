
use serde::Deserialize;

// structure only used to obtain useful data from the registers
// when reading tyhe csv data file
#[derive(Debug, Deserialize)]
pub struct ReviewInfo {
    pub app_id: u32,
    pub app_name: String,
    pub language: String,
    pub review: String,
    pub votes_helpful: u64
}
