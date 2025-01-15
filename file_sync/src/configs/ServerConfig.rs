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
}

impl ServerConfig {
    #[doc = "감시하는 파일 리스트를 반환해주는 함수"]
    pub fn get_watch_file_list(&self) -> Vec<String> {
        let watch_path: String = self.watch_path.clone();

        let watch_file_lists = self
            .specific_files
            .iter()
            .map(|file| format!("{}{}", watch_path, file))
            .collect::<Vec<String>>();

        watch_file_lists
    }
}
