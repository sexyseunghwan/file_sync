use crate::common::*;

#[derive(Deserialize, Debug, Getters, new)]
#[getset(get = "pub")]
pub struct MonitoringPathInfo {
    pub file_path: String,
    pub full_file_path: String,
}
