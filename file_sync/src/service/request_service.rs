use crate::common::*;

use crate::configs::Configs::*;

use crate::utils_modules::request_utils::*;


#[async_trait]
pub trait RequestService {
    async fn send_info_to_slave(&self, file_path: &str) -> Result<(), anyhow::Error>;
    async fn send_info_to_slave_io(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error>;
    async fn send_info_to_slave_memory(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error>;
    fn handle_async_function(&self, task_res: Vec<Result<Result<(), anyhow::Error>, task::JoinError>>) -> Result<(), anyhow::Error>;
}


#[derive(Debug)]
pub struct RequestServicePub {
    pub client: Arc<Client>  
}

impl RequestServicePub { 
    
    pub fn new() -> Self {

        let client = Arc::new(Client::new());
        
        Self { client }

    }   
    
}


#[async_trait]
impl RequestService for RequestServicePub {

    
    #[doc = "master server 에서 파일이 변경되는 경우 해당 변경 정보를 slave server에 공유해준다."]
    async fn send_info_to_slave(&self, file_path: &str) -> Result<(), anyhow::Error> {

        let slave_url;
        let io_improvement_option;
        let file_name;

        {
            let server_config = get_config_read()?;
            slave_url = server_config
                .server
                .slave_address()
                .as_ref()
                .cloned() // 데이터를 복제하여 잠금 외부로 이동
                .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] 'slave_url' not found."))?;

            let path = Path::new(file_path);
            file_name = path.file_name()
                .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] The file name is not valid."))?
                .to_str()
                .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] There was a problem converting the file name to a string."))?
                .to_owned();  // String으로 복제하여 잠금 외부로 이동

            io_improvement_option = *server_config.server.io_bound_improvement();
        } 
        
        // let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;

        // let slave_url = server_config
        //     .server
        //     .slave_address()
        //     .as_ref()
        //     .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] 'slave_url' not found."))?;

        // let path = Path::new(file_path);
        // let file_name = path.file_name()
        //     .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] The file name is not valid."))?
        //     .to_str()
        //     .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] There was a problem converting the file name to a string."))?;

        // /* i/o 를 효율화 할지 메므리를 효율화 할지 정해주는 변수 */
        // let io_improvement_option = server_config.server.io_bound_improvement();
        
        if io_improvement_option {
            self.send_info_to_slave_io(file_path, &file_name, slave_url.clone()).await?;
        } else {
            self.send_info_to_slave_memory(file_path, &file_name, slave_url.clone()).await?;
        }
        
        Ok(())
    }
    
    

    #[doc = "i/o bound 효율코드"]
    async fn send_info_to_slave_io(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error> {

        let file_data = tokio::fs::read(&file_path).await?;
        let from_host;
        {
            let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;
            from_host = server_config.server.host().to_string();
        }
        
        let tasks: Vec<_> = slave_url.into_iter().map(|url: String| {
                
            let client  = self.client.clone();
            let data_clone = file_data.clone();
            let parsing_url = format!("http://{}/upload?filename={}", url, file_name);
            let file_path = file_path.to_string().clone();  
            let from_host_clone = from_host.clone();
            
            task::spawn(async move {
                let from_host_move_clone = from_host_clone.clone();
                send_file_to_url(&client, &parsing_url, &data_clone, &file_path, &from_host_move_clone, &url).await
            })

        }).collect();
        
        let results: Vec<Result<Result<(), anyhow::Error>, task::JoinError>> = join_all(tasks).await;
        self.handle_async_function(results)
    }
    
    

    #[doc = "메모리 효율 코드"]
    async fn send_info_to_slave_memory(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error>{

        let from_host;
        {
            let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;
            from_host = server_config.server.host().to_string();
        }

        let tasks: Vec<_> = slave_url.into_iter().map(|url| {
                
            let client = self.client.clone();
            let parsing_url = format!("http://{}/upload?filename={}", url, file_name);
            let file_path = file_path.to_string().clone();
            let from_host_clone = from_host.clone();

            task::spawn(async move {
                let file_data = tokio::fs::read(&file_path).await?;
                let from_host_move_clone = from_host_clone.clone();
                send_file_to_url(&client, &parsing_url, &file_data, &file_path, &from_host_move_clone, &url).await
            })

        }).collect();

        let results: Vec<Result<Result<(), anyhow::Error>, task::JoinError>> = join_all(tasks).await;
        self.handle_async_function(results)
    }
    
    
    
    #[doc = "async 함수들의 결과를 파싱해주는 함수"]
    fn handle_async_function(&self, task_res: Vec<Result<Result<(), anyhow::Error>, task::JoinError>>) -> Result<(), anyhow::Error> {

        let mut all_good = true;
        
        for result in task_res {
            match result {
                Ok(Ok(())) => continue,  // Success
                Ok(Err(e)) => {
                    error!("Task failed with error: {}", e);
                    all_good = false;
                },
                Err(e) => {
                    // This is the case where the spawned task panicked or couldn't be executed
                    error!("Task panicked or couldn't be executed: {}", e);
                    all_good = false;
                }
            }
        }
        
        if all_good {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Some tasks failed"))
        }
    }

}

// #[doc = "HTTP 요청을 처리 해주는 함수 - 수정 파일배포 관련 함수"]
// pub async fn send_file_to_url(
//     client: &Client, 
//     url: &str, 
//     file_data: &[u8], 
//     file_path: &str,
//     from_host: &str,
//     to_host: &str
// ) -> Result<(), anyhow::Error> {
    
//     let body = Body::from(file_data.to_vec());
    
//     let response = client.post(url)
//         .header("Content-Type", "multipart/form-data")
//         .body(body)
//         .send()
//         .await?;
    
//     if response.status().is_success() {
//         info!("File was sent successfully: {}", url);
//         let es_msg = ElasticMsg::new(from_host, to_host, file_path, "success", "master task")?;
//         send_task_message_to_elastic(es_msg).await?;
//         Ok(())
//     } else {
//         let es_msg = ElasticMsg::new(from_host, to_host, file_path, "failed", "master task")?;
//         send_task_message_to_elastic(es_msg).await?;
//         Err(anyhow!("[Error] Failed to send file: {}", response.status()))
//     }
// }


// #[doc = "파일 공유 작업 관련 메시지를 elasticsearch 'file_sync_log' 로그에 남겨주는 함수"]
// pub async fn send_task_message_to_elastic<T: Serialize + Sync + Send>(json_data: T) -> Result<(), anyhow::Error> {

//     let es_conn = get_elastic_conn();
//     let data_json = serde_json::to_value(json_data)?;

//     let cur_date_utc = get_current_utc_naivedate_str("%Y%m%d")?;
//     let index_name = format!("file_sync_log_{}", cur_date_utc);

//     es_conn.post_doc(&index_name, data_json).await?;

//     Ok(())
// }