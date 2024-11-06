use crate::common::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub role: String,
    pub master_address: Option<String>,
    pub slave_address: Option<Vec<String>>,
    pub watch_path: String,
    pub specific_files: Vec<String>,
}