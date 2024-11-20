use crate::common::*;

use crate::utils_modules::time_utils::*;

use crate::model::ElasticMsg::*;

use crate::repository::elastic_repository::*;



#[doc = "HTTP 요청을 처리 해주는 함수 - 수정 파일배포 관련 함수"]
/// # Arguments 
/// * `client`      - Client 객체
/// * `url`         - 요청(request)대상이 되는 서버의 url
/// * `file_data`   - 파일 스트림 데이터
/// * `file_path`   - 대상 파일
/// * `from_host`   - 요청(request)을 보내는 호스트 주소
/// * `to_host`     - 요청(request)을 받는 호스트 주소
/// 
/// # Returns
/// * Result<(), anyhow::Error>
pub async fn send_file_to_url(
    client: &Client, 
    url: &str, 
    file_data: &[u8], 
    file_path: &str,
    from_host: &str,
    to_host: &str
) -> Result<(), anyhow::Error> {
    
    let body = Body::from(file_data.to_vec());
    
    let response = client.post(url)
        .header("Content-Type", "multipart/form-data")
        .body(body)
        .send()
        .await?;
    
    if response.status().is_success() {
        info!("File was sent successfully: {}", url);
        let es_msg = ElasticMsg::new(from_host, to_host, file_path, "success", "master task")?;
        send_task_message_to_elastic(es_msg).await?;
        Ok(())
    } else {
        let es_msg = ElasticMsg::new(from_host, to_host, file_path, "failed", "master task")?;
        send_task_message_to_elastic(es_msg).await?;
        Err(anyhow!("[Error] Failed to send file: {}", response.status()))
    }
}



#[doc = "라우터 함수에서 진행된 작업에 대한 로그를 Elasticsearch 로 보내주기 위한 함수"]
/// # Arguments
/// * `from_host`   - 작업진행 서버 주소
/// * `to_host`     - 피작업 진행 서버 주소
/// * `file_path`   - 수정된 파일 절대경로
/// * `task_status` - 작업 성공/실패 여부
/// * `task_detail` - 작업 관련 디테일 메시지
/// 
/// # Returns
/// * Result<(), anyhow::Error>
pub async fn post_log_to_es(from_host: &str, to_host: &str, file_path: &str, task_status: &str, task_detail: &str) -> Result<(), anyhow::Error> {
    
    let es_msg = ElasticMsg::new(
        from_host, 
        to_host, 
        file_path, 
        task_status, 
        task_detail)?;
    
    send_task_message_to_elastic(es_msg).await?;

    Ok(())
}



#[doc = "파일 공유 작업 관련 메시지를 elasticsearch 'file_sync_log' 로그에 남겨주는 함수"]
/// # Arguments
/// * `json_data` - Elasticsearch 로 보낼 json 객체
/// 
/// # Returns
/// * Result<(), anyhow::Error>
async fn send_task_message_to_elastic<T: Serialize + Sync + Send>(json_data: T) -> Result<(), anyhow::Error> {

    let es_conn = get_elastic_conn();
    let data_json = serde_json::to_value(json_data)?;

    let cur_date_utc = get_current_utc_naivedate_str("%Y%m%d")?;
    let index_name = format!("file_sync_log_{}", cur_date_utc);

    es_conn.post_doc(&index_name, data_json).await?;

    Ok(())
}