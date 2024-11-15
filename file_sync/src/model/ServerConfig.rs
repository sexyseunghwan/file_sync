use crate::common::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub role: String,
    pub master_address: Option<Vec<String>>,
    pub slave_address: Option<Vec<String>>,
    pub slave_host: Option<String>,
    pub watch_path: String,
    pub specific_files: Vec<String>,
    pub io_bound_improvement: bool,
    pub slave_backup_path: Option<String>
}