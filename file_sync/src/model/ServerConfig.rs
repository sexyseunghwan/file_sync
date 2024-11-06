use crate::common::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    role: String,
    master_address: Option<String>,
    slave_address: Option<Vec<String>>,
    watch_path: String,
    specific_files: Vec<String>,
}