use crate::common::*;

use crate::utils_modules::tls_utils::*;

use hyper::Request;
use hyper_rustls::{HttpsConnectorBuilder, HttpsConnector};
use hyper_util::{
    rt::TokioExecutor, 
    client::legacy::Client,
    client::legacy::connect::HttpConnector
};
use http_body_util::Full;
use bytes::Bytes;
use rustls::ClientConfig;

static SECURE_FILE_REQ_CLIENT: once_lazy<Arc<SecureFileTransferClient>> =
    once_lazy::new(initialize_secure_request_clients);

pub fn initialize_secure_request_clients() -> Arc<SecureFileTransferClient> {
    match SecureFileTransferClient::new() {
        Ok(client) => Arc::new(client),
        Err(e) => {
            error!("Failed to initialize secure client: {}", e);
            panic!("Cannot initialize secure client");
        }
    }
}

pub fn get_secure_request_client() -> Arc<SecureFileTransferClient> {
    let req_client: &once_lazy<Arc<SecureFileTransferClient>> = &SECURE_FILE_REQ_CLIENT;
    Arc::clone(req_client)
}

#[derive(Debug, Clone)]
pub struct SecureFileTransferClient {
    client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
}

impl SecureFileTransferClient {
    pub fn new() -> Result<Self, anyhow::Error> {
        let tls_config: ClientConfig = create_client_tls_config()?;
        
        let https: HttpsConnector<HttpConnector> = HttpsConnectorBuilder::new()
            .with_tls_config(tls_config)
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();

        let client: Client<HttpsConnector<HttpConnector>, Full<Bytes>> = Client::builder(TokioExecutor::new()).build(https);

        Ok(Self { client })
    }

    #[doc = "HTTPS 요청을 처리 해주는 함수 - 수정 파일배포 관련 함수"]
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
        let body: Full<Bytes> = Full::new(Bytes::from(file_data.to_vec()));

        let req: Request<Full<Bytes>> = Request::post(url)
            .header("Content-Type", "multipart/form-data")
            .header("X-File-Path", file_path)
            .header("X-From-Host", from_host)
            .header("X-To-Host", to_host)
            .body(body)?;

        let response: hyper::Response<hyper::body::Incoming> = self.client.request(req).await?;

        if response.status().is_success() {
            info!(
                "Secure file transfer successful: {} // file_path: {} // from_host: {} // to_host: {}",
                url, file_path, from_host, to_host
            );
            Ok(())
        } else {
            Err(anyhow!(
                "[ERROR][SecureFileTransferClient->send_file_to_url] Secure file transfer failed: {} // {} // file_path: {} // from_host: {} // to_host: {}",
                response.status(),
                url, file_path, from_host, to_host
            ))
        }
    }
}