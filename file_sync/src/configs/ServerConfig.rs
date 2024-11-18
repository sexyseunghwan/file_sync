use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters)]
#[getset(get = "pub", set = "pub")]
pub struct ServerConfig {
    pub role: String,
    pub host: String,
    pub master_address: Option<Vec<String>>,
    pub slave_address: Option<Vec<String>>,
    pub watch_path: String,
    pub specific_files: Vec<String>,
    pub io_bound_improvement: bool,
    pub slave_backup_path: Option<String>,
    pub elastic_host: Option<Vec<String>>,
    pub elastic_id: Option<String>,
    pub elastic_pw: Option<String>
}