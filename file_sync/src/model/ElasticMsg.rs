use crate::common::*;

use crate::utils_modules::time_utils::*;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ElasticMsg {
    pub timestamp: String,
    pub from_host: String,
    pub to_host: String,
    pub file_path: String,
    pub task_status: String,
    pub task_detail: String
}


impl ElasticMsg {
    
    #[doc = "ElasticMsg의 생성자"]
    /// # Arguments
    /// * `from_host`   - 작업진행 서버 주소
    /// * `to_host`     - 피작업 진행 서버 주소
    /// * `file_path`   - 수정된 파일 절대경로
    /// * `task_status` - 작업 성공/실패 여부
    /// * `task_detail` - 작업 관련 디테일 메시지
    /// 
    /// 
    /// # Returns
    /// * Result<Self, anyhow::Error>
    pub fn new(from_host: &str, to_host: &str, file_path: &str, task_status: &str, task_detail: &str) -> Result<Self, anyhow::Error> {

        let curr_time = get_current_utc_naivedatetime_str("%Y-%m-%dT%H:%M:%SZ")?;
        
        let elastic_msg = ElasticMsg {
            timestamp: curr_time,
            from_host: from_host.to_string(),
            to_host: to_host.to_string(),
            file_path: file_path.to_string(), 
            task_status: task_status.to_string(), 
            task_detail: task_detail.to_string()
        };
        
        Ok(elastic_msg)
    }

}