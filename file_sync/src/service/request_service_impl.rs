use crate::common::*;

use crate::configs::configs::*;

use crate::traits::service::request_service::*;

use crate::external_clients::file_transfer_client::*;
use crate::external_clients::secure_file_transfer_client::*;

#[derive(Debug, new)]
pub struct RequestServiceImpl;

#[async_trait]
impl RequestService for RequestServiceImpl {
    #[doc = "master server 에서 파일이 변경되는 경우 해당 변경 정보를 slave server에 공유해준다."]
    /// # Arguments
    /// * `file_path` - 수정된 파일경로
    /// * `file_name` - 수정된 파일이름
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_info_to_slave(
        &self,
        file_path: &str,
        file_name: &str,
    ) -> Result<(), anyhow::Error> {
        let slave_url: Vec<String>;
        let io_improvement_option: bool; /* io 효율코드 옵션 적용 유무 */
        let secure_mode: bool; /* 보안모드 적용 유무 */
        {
            let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;
            slave_url = server_config
                .server
                .slave_address()
                .clone()
                .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] 'slave_url' not found."))?;

            io_improvement_option = *server_config.server.io_bound_improvement();
            secure_mode = server_config.server.is_secure_mode();
        }

        if io_improvement_option {
            self.send_info_to_slave_io(file_path, file_name, slave_url.clone(), secure_mode)
                .await?;
        } else {
            /* io 효율코드 옵션을 적용하지 않으면 메모리 효율코드 옵션이 지정된다. */
            self.send_info_to_slave_memory(file_path, file_name, slave_url.clone(), secure_mode)
                .await?;
        }

        Ok(())
    }
    
    #[doc = "i/o bound 효율코드"]
    /// # Arguments
    /// * `file_path` - 수정된 파일 경로
    /// * `file_name` - 수정된 파일 이름
    /// * `slave_url` - 동기화 대상이 되는 서버들
    /// * `secure_mode` - TLS 사용 여부
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_info_to_slave_io(
        &self,
        file_path: &str,
        file_name: &str,
        slave_url: Vec<String>,
        secure_mode: bool,
    ) -> Result<(), anyhow::Error> {
        /* 변경된 파일의 데이터를 read 하여 메모리에 상주시킨다. */
        let file_data: Vec<u8> = tokio::fs::read(&file_path).await?;
        let from_host: String;
        {
            let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;
            from_host = server_config.server.host().to_string();
        }
        
        /* Slave Server 는 지금 Actix-web 으로 작동되고 있으므로 api 형식을 사용할때처럼 데이터를 송신해주는 것. */
        let tasks: Vec<_> = slave_url
            .into_iter()
            .map(|url: String| {
                let data_clone: Vec<u8> = file_data.clone(); /* 변경된 파일 데이터 복제: 소유권으로 인한 문제 */
                let protocol: &str = if secure_mode { "https" } else { "http" };
                let parsing_url: String = format!("{}://{}/upload?filename={}", protocol, url, file_name);
                let file_path: String = file_path.to_string().clone();
                let from_host_clone: String = from_host.clone();

                task::spawn(async move {
                    let from_host_move_clone: String = from_host_clone.clone();
                    
                    if secure_mode {
                        let req_repo: Arc<SecureFileTransferClient> = get_secure_request_client();
                        req_repo
                            .send_file_to_url(
                                &parsing_url,
                                &data_clone,
                                &file_path,
                                &from_host_move_clone,
                                &url,
                            )
                            .await
                    } else {
                        let req_repo: Arc<FileTransferClient> = get_request_client();
                        req_repo
                            .send_file_to_url(
                                &parsing_url,
                                &data_clone,
                                &file_path,
                                &from_host_move_clone,
                                &url,
                            )
                            .await
                    }
                })
            })
            .collect();

        let results: Vec<Result<Result<(), anyhow::Error>, task::JoinError>> =
            join_all(tasks).await;
        self.handle_async_function(results)
    }
    
    #[doc = "메모리 효율 코드"]
    /// # Arguments
    /// * `file_path` - 수정된 파일 경로
    /// * `file_name` - 수정된 파일 이름
    /// * `slave_url` - 동기화 대상이 되는 서버들
    /// * `secure_mode` - TLS 사용 여부
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_info_to_slave_memory(
        &self,
        file_path: &str,
        file_name: &str,
        slave_url: Vec<String>,
        secure_mode: bool,
    ) -> Result<(), anyhow::Error> {
        let from_host: String;
        {
            let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;
            from_host = server_config.server.host().to_string();
        }

        let tasks: Vec<_> = slave_url
            .into_iter()
            .map(|url| {
                let protocol: &str = if secure_mode { "https" } else { "http" };
                let parsing_url: String = format!("{}://{}/upload?filename={}", protocol, url, file_name);
                let file_path: String = file_path.to_string().clone();
                let from_host_clone: String = from_host.clone();

                task::spawn(async move {
                    let file_data: Vec<u8> = tokio::fs::read(&file_path).await?;
                    let from_host_move_clone: String = from_host_clone.clone();
                    
                    if secure_mode {
                        let req_repo: Arc<SecureFileTransferClient> = get_secure_request_client();
                        req_repo
                            .send_file_to_url(
                                &parsing_url,
                                &file_data,
                                &file_path,
                                &from_host_move_clone,
                                &url,
                            )
                            .await
                    } else {
                        let req_repo: Arc<FileTransferClient> = get_request_client();
                        req_repo
                            .send_file_to_url(
                                &parsing_url,
                                &file_data,
                                &file_path,
                                &from_host_move_clone,
                                &url,
                            )
                            .await
                    }
                })
            })
            .collect();

        let results: Vec<Result<Result<(), anyhow::Error>, task::JoinError>> =
            join_all(tasks).await;

        self.handle_async_function(results)
    }

    #[doc = "async 함수들의 결과를 파싱해주는 함수"]
    /// # Arguments
    /// * `task_res` - 비동기 함수의 결과
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    fn handle_async_function(
        &self,
        task_res: Vec<Result<Result<(), anyhow::Error>, task::JoinError>>,
    ) -> Result<(), anyhow::Error> {
        let mut all_good: bool = true;

        for result in task_res {
            match result {
                Ok(Ok(())) => continue, /* Success */
                Ok(Err(e)) => {
                    error!(
                        "[Error][handle_async_function()] Task failed with error: {:?}",
                        e
                    );
                    all_good = false;
                }
                Err(e) => {
                    /* This is the case where the spawned task panicked or couldn't be executed */
                    error!("[Error][handle_async_function()] Task panicked or couldn't be executed: {}", e);
                    all_good = false;
                }
            }
        }

        if all_good {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "[Error][handle_async_function()] Some tasks failed"
            ))
        }
    }
}
