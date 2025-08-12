use crate::common::*;

static FILE_REQ_CLIENT: once_lazy<Arc<FileTransferClient>> =
    once_lazy::new(initialize_request_clients);

pub fn initialize_request_clients() -> Arc<FileTransferClient> {
    let client: Client = Client::new();
    Arc::new(FileTransferClient::new(client))
}

pub fn get_request_client() -> Arc<FileTransferClient> {
    let req_client: &once_lazy<Arc<FileTransferClient>> = &FILE_REQ_CLIENT;
    Arc::clone(req_client)
}

#[derive(Debug, Getters, Clone, new)]
pub struct FileTransferClient {
    pub client: Client,
}

impl FileTransferClient {
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
    pub async fn send_file_to_url(
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
            info!(
                "File was sent successfully: {} // file_path: {} // from_host: {} // to_host: {}",
                url, file_path, from_host, to_host
            );
            Ok(())
        } else {
            Err(anyhow!(
                "[Error] Failed to send file: {} // {} // file_path: {} // from_host: {} // to_host: {}",
                response.status(),
                url, file_path, from_host, to_host
            ))
        }
    }
}
