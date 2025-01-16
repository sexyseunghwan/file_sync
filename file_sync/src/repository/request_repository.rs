use crate::common::*;

use crate::model::ElasticMsg::*;

use crate::repository::elastic_repository::*;

use crate::utils_modules::time_utils::*;

#[doc = ""]
static REQ_CLIENT: once_lazy<Arc<ReqRepositoryPub>> =
    once_lazy::new(|| initialize_request_clients());

#[doc = ""]
pub fn initialize_request_clients() -> Arc<ReqRepositoryPub> {
    let client: Client = Client::new();
    Arc::new(ReqRepositoryPub::new(client))
}

#[doc = ""]
pub fn get_request_client() -> Arc<ReqRepositoryPub> {
    let req_client: &once_lazy<Arc<ReqRepositoryPub>> = &REQ_CLIENT;
    Arc::clone(req_client)
}

#[async_trait]
pub trait ReqRepository {
    async fn send_file_to_url(
        &self,
        url: &str,
        file_data: &[u8],
        file_path: &str,
        from_host: &str,
        to_host: &str,
    ) -> Result<(), anyhow::Error>;
    async fn send_task_message_to_elastic<T: Serialize + Sync + Send>(
        &self,
        json_data: T,
    ) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Getters, Clone, new)]
pub struct ReqRepositoryPub {
    pub client: Client,
}

#[async_trait]
impl ReqRepository for ReqRepositoryPub {
    #[doc = "HTTP 요청을 처리 해주는 함수 - 수정 파일배포 관련 함수"]
    /// # Arguments
    /// * `url`         - 요청(request)대상이 되는 서버의 url
    /// * `file_data`   - 파일 스트림 데이터
    /// * `file_path`   - 대상 파일
    /// * `from_host`   - 요청(request)을 보내는 호스트 주소
    /// * `to_host`     - 요청(request)을 받는 호스트 주소
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_file_to_url(
        &self,
        url: &str,
        file_data: &[u8],
        file_path: &str,
        from_host: &str,
        to_host: &str,
    ) -> Result<(), anyhow::Error> {
        let body: Body = Body::from(file_data.to_vec());

        let response: reqwest::Response = self
            .client
            .post(url)
            .header("Content-Type", "multipart/form-data")
            .body(body)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("File was sent successfully: {}", url);
            let es_msg: ElasticMsg = ElasticMsg::new(from_host, to_host, file_path, "success", "master task")?;
            self.send_task_message_to_elastic(es_msg).await?;
            Ok(())
        } else {
            let es_msg: ElasticMsg = ElasticMsg::new(from_host, to_host, file_path, "failed", "master task")?;
            self.send_task_message_to_elastic(es_msg).await?;
            Err(anyhow!(
                "[Error] Failed to send file: {}",
                response.status()
            ))
        }
    }
    
    #[doc = "파일 공유 작업 관련 메시지를 elasticsearch 'file_sync_log' 로그에 남겨주는 함수"]
    /// # Arguments
    /// * `json_data` - Elasticsearch 로 보낼 json 객체
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_task_message_to_elastic<T: Serialize + Sync + Send>(
        &self,
        json_data: T,
    ) -> Result<(), anyhow::Error> {
        let es_conn: Arc<EsRepositoryPub> = get_elastic_conn();
        let data_json: Value = serde_json::to_value(json_data)?;

        let cur_date_utc: String = get_current_utc_naivedate_str("%Y%m%d")?;
        let index_name: String = format!("test_sync_log_{}", cur_date_utc);

        es_conn.post_doc(&index_name, data_json).await?;

        Ok(())
    }
}
