use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters)]
#[getset(get = "pub")]
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
    pub elastic_pw: Option<String>,
    pub backup_days: Option<i64>,
    pub hash_storage_path: Option<String>,
    pub secure_mode: Option<bool>,
    pub key_directory: Option<String>,
}

impl ServerConfig {
    #[doc = "감시하는 디렉토리"]
    pub fn get_watch_dir_path(&self) -> String {
        self.watch_path().clone()
    }

    #[doc = "보안 모드 활성화 여부"]
    pub fn is_secure_mode(&self) -> bool {
        match self.secure_mode() {
            Some(val) => *val,
            None => false,
        }
    }

    #[doc = "키 디렉토리 경로"]
    pub fn get_key_directory(&self) -> String {
        self.key_directory()
            .as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| "keys".to_string())
    }
}
