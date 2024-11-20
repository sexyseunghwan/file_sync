use crate::common::*;

use crate::utils_modules::time_utils::*;

use crate::model::ElasticMsg::*;

use crate::repository::elastic_repository::*;


#[doc = "HTTP 요청을 처리 해주는 함수 - 수정 파일배포 관련 함수"]
/// # Arguments 
/// * `client`      -
/// * `url`         -
/// * `file_data`   -
/// * `file_path`   -
/// * `from_host`   -
/// * `to_host`     -
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


#[doc = "파일 공유 작업 관련 메시지를 elasticsearch 'file_sync_log' 로그에 남겨주는 함수"]
/// # Arguments
/// * `json_data` - 
/// 
/// # Returns
/// * Result<(), anyhow::Error>
pub async fn send_task_message_to_elastic<T: Serialize + Sync + Send>(json_data: T) -> Result<(), anyhow::Error> {

    let es_conn = get_elastic_conn();
    let data_json = serde_json::to_value(json_data)?;

    let cur_date_utc = get_current_utc_naivedate_str("%Y%m%d")?;
    let index_name = format!("file_sync_log_{}", cur_date_utc);

    es_conn.post_doc(&index_name, data_json).await?;

    Ok(())
}