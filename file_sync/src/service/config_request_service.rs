use crate::common::*;

use crate::model::Configs::*;

use crate::utils_modules::io_utils::*;


#[async_trait]
pub trait ConfigRequestService {
    fn get_role(&self) -> String;
    fn get_slave_address(&self) -> Result<Vec<String>, anyhow::Error>;
    fn get_watch_file_list(&self) -> Vec<String>;
    fn get_watch_dir_info(&self) -> String;
    fn get_slave_host(&self) -> Result<String, anyhow::Error>;
    fn get_master_address(&self) -> Result<Vec<String>, anyhow::Error>;
    fn get_slave_backup_path(&self) -> Result<String, anyhow::Error>; 

    async fn send_info_to_slave(&self, file_path: &str) -> Result<(), anyhow::Error>;
    async fn send_info_to_slave_io(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error>;
    async fn send_info_to_slave_memory(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error>;
    fn handle_async_function(&self, task_res: Vec<Result<Result<(), anyhow::Error>, task::JoinError>>) -> Result<(), anyhow::Error>;
}


#[derive(Debug)]
pub struct ConfigRequestServicePub {
    pub config: Configs,
    pub client: Client
}


#[derive(Debug, Deserialize, Serialize, new)]
pub struct RequestReturnMessage {
    pub err_flag: bool,
    pub err_msg: String
}


impl ConfigRequestServicePub {
    
    pub fn new() -> Self {

        let config: Configs = match read_toml_from_file::<Configs>("./Config.toml") {
            Ok(config) => config,
            Err(e) => {
                error!("[Error][WatchServicePub->new()]{:?}", e);
                panic!("{:?}", e)
            }
        };
        
        let client = Client::new();

        Self { config, client }
    }    
}

#[async_trait]
impl ConfigRequestService for ConfigRequestServicePub {
    
    #[doc = "slave_backup_path 정보를 가져와주는 함수"]
    fn get_slave_backup_path(&self) -> Result<String, anyhow::Error> {

        let slave_backup_path = self
            .config
            .server
            .slave_backup_path
            .clone()
            .ok_or_else(|| anyhow!("[Error][get_slave_backup_path()] There was a problem processing information 'slave_backup_path'."))?;

        Ok(slave_backup_path)
    }
    
    #[doc = "master_address 정보를 가져와주는 함수"]
    fn get_master_address(&self) -> Result<Vec<String>, anyhow::Error> {

        let master_address: Vec<String> = self
            .config
            .server
            .master_address
            .clone()
            .ok_or_else(|| anyhow!("[Error][get_master_address()]  There was a problem processing information 'master_address'."))?;

        Ok(master_address)
    }
    
    #[doc = "slave_host 정보를 가져와주는 함수"]
    fn get_slave_host(&self) -> Result<String, anyhow::Error> {
        
        let slave_host = self
            .config
            .server
            .slave_host
            .clone()
            .ok_or_else(|| anyhow!("[Error][get_slave_host()] There was a problem processing information 'slave_host'."))?;
        
        Ok(slave_host)
    }   
    
    #[doc = "감시대상 디렉토리 경로를 반환해주는 함수"]
    fn get_watch_dir_info(&self) -> String {
        self.config.server.watch_path.clone()
    }


    #[doc = "해당 프로그램의 역할을 조회해주는 함수"]
    fn get_role(&self) -> String {
        self.config.server.role.clone()
    }
    
    
    #[doc = "slave_address 정보를 반환하는 함수"]
    fn get_slave_address(&self) -> Result<Vec<String>, anyhow::Error> {

        let slave_address_vec = match self.config.server.slave_address.clone() {
            Some(slave_address_vec) => slave_address_vec,
            None => {
                return Err(anyhow!("[Error][get_slave_address()] "));
            }
        };

        Ok(slave_address_vec)
    }


    #[doc = "감시하는 파일 리스트를 반환해주는 함수"]
    fn get_watch_file_list(&self) -> Vec<String> {

        let watch_path: String = self.config.server.watch_path.clone();

        let watch_file_lists = self.config.server.specific_files
            .iter()
            .map(|file| format!("{}{}", watch_path, file))
            .collect::<Vec<String>>();

        watch_file_lists
        
    }
    
    #[doc = "async 함수들의 결과를 파싱해주는 함수"]
    fn handle_async_function(&self, task_res: Vec<Result<Result<(), anyhow::Error>, task::JoinError>>) -> Result<(), anyhow::Error> {

        let mut all_good = true;

        for result in task_res {
            match result {
                Ok(Ok(())) => continue,  // Success path
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

    #[doc = "master 에서 파일이 변경되는 경우 해당 변경 정보를 slave 서버에 보내준다."]
    async fn send_info_to_slave(&self, file_path: &str) -> Result<(), anyhow::Error> {
        
        println!("file_path= {:?}", file_path);

        let slave_url = self
            .config
            .server
            .slave_address
            .as_ref()
            .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] 'slave_url' not found."))?;

        println!("send_info_to_slave= {:?}", slave_url);

        let path = Path::new(file_path);
        let file_name = path.file_name()
            .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] The file name is not valid."))?
            .to_str()
            .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] There was a problem converting the file name to a string."))?;

        
        let io_improvement_option = self.config.server.io_bound_improvement;
        
        if io_improvement_option {
            self.send_info_to_slave_io(file_path, file_name, slave_url.clone()).await?;
        } else {
            self.send_info_to_slave_memory(file_path, file_name, slave_url.clone()).await?;
        }

        Ok(())
    }
    
    
    #[doc = "i/o 바운드 효율코드"]
    async fn send_info_to_slave_io(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error> {

        let file_data = tokio::fs::read(file_path).await?;
        
        let tasks: Vec<_> = slave_url.into_iter().map(|url| {
            
            let client = self.client.clone();
            let data_clone = file_data.clone();
            let parsing_url = format!("http://{}/upload?filename={}", url, file_name);

            println!("parsing_url=  {:?}",parsing_url);

            task::spawn(async move {
                
                let body = Body::from(data_clone);
                let response = client.post(&parsing_url)
                    .header("Content-Type", "multipart/form-data")
                    .body(body)
                    .send()
                    .await;
                
                match response {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            info!("File was sent successfully: {}", &url);
                            Ok(())
                        } else {
                            Err(anyhow!("[Error][send_info_to_slave_io()] Failed to send file: {}", resp.status()))
                        }
                    },
                    Err(e) => Err(anyhow!("[Error][send_info_to_slave_io()] HTTP request failed: {}", e)),
                }
            })
        }).collect();

        let results: Vec<Result<Result<(), anyhow::Error>, task::JoinError>> = join_all(tasks).await;
        self.handle_async_function(results)

    }
    

    #[doc = "메모리 효율코드"]
    async fn send_info_to_slave_memory(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error> {
        
        let tasks: Vec<_> = slave_url.into_iter().map(|url| {
                
            let client = self.client.clone();
            let parsing_url = format!("http://{}/upload?filename={}", url, file_name);
            let file_path = file_path.to_string().clone();

            task::spawn(async move {
                
                let file_data = tokio::fs::read(file_path).await?;
                let body = Body::from(file_data);
                let response = client.post(&parsing_url)
                    .header("Content-Type", "multipart/form-data")
                    .body(body)
                    .send()
                    .await;
                
                match response {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            info!("File was sent successfully: {}", &url);
                            Ok(())
                        } else {
                            Err(anyhow!("[Error][send_info_to_slave_io()] Failed to send file: {}", resp.status()))
                        }
                    },
                    Err(e) => Err(anyhow!("[Error][send_info_to_slave_io()] HTTP request failed: {}", e)),
                }
            })
        }).collect();

        let results: Vec<Result<Result<(), anyhow::Error>, task::JoinError>> = join_all(tasks).await;
        self.handle_async_function(results)
    }
}