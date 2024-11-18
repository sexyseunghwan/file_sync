use crate::common::*;

use crate::model::Configs::*;

use crate::utils_modules::io_utils::*;
use crate::utils_modules::request_utils::*;


#[async_trait]
pub trait ConfigRequestService {
    
    /* Config 부분 */
    fn get_role(&self) -> String;
    fn get_watch_file_list(&self) -> Vec<String>;
    fn get_watch_dir_info(&self) -> String;
    fn get_host_info(&self) -> String;
    fn get_master_address(&self) -> Result<Vec<String>, anyhow::Error>;
    fn get_slave_backup_path(&self) -> Result<String, anyhow::Error>; 
    
    /* Request 부분 */
    async fn send_info_to_slave(&self, file_path: &str) -> Result<(), anyhow::Error>;
    async fn send_info_to_slave_io(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error>;
    async fn send_info_to_slave_memory(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error>;
    fn handle_async_function(&self, task_res: Vec<Result<Result<(), anyhow::Error>, task::JoinError>>) -> Result<(), anyhow::Error>;
}


#[derive(Debug)]
pub struct ConfigRequestServicePub {
    pub config: Configs,        
    pub client: Arc<Client>     
}


impl ConfigRequestServicePub {
    
    pub fn new() -> Self {  
        
        let config: Configs = match read_toml_from_file::<Configs>("./config/Config.toml") {
            Ok(config) => config,
            Err(e) => {
                error!("[Error][WatchServicePub->new() ???]{:?}", e);
                panic!("{:?}", e)
            }
        };
        
        let client = Arc::new(Client::new());

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
    
    #[doc = "프로그램을 동작 주체가 되는 host 정보를 가져와주는 함수"]
    fn get_host_info(&self) -> String {
        
        let slave_host = self
            .config
            .server
            .host
            .clone();
        
        slave_host
    }   
    
    #[doc = "감시대상 디렉토리 경로를 반환해주는 함수"]
    fn get_watch_dir_info(&self) -> String {
        self.config.server.watch_path.clone()
    }
    

    #[doc = "해당 프로그램의 역할을 조회해주는 함수"]
    fn get_role(&self) -> String {
        self.config.server.role.clone()
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


    #[doc = "master server 에서 파일이 변경되는 경우 해당 변경 정보를 slave server에 공유해준다."]
    async fn send_info_to_slave(&self, file_path: &str) -> Result<(), anyhow::Error> {
        
        let slave_url = self
            .config
            .server
            .slave_address
            .as_ref()
            .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] 'slave_url' not found."))?;
        
        let path = Path::new(file_path);
        let file_name = path.file_name()
            .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] The file name is not valid."))?
            .to_str()
            .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] There was a problem converting the file name to a string."))?;
        
        /* i/o 를 효율화 할지 메므리를 효율화 할지 정해주는 변수 */
        let io_improvement_option = self.config.server.io_bound_improvement;
        
        if io_improvement_option {
            self.send_info_to_slave_io(file_path, file_name, slave_url.clone()).await?;
        } else {
            self.send_info_to_slave_memory(file_path, file_name, slave_url.clone()).await?;
        }

        Ok(())
    }
    
    
    #[doc = "i/o bound 효율코드"]
    async fn send_info_to_slave_io(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error> {
        
        let file_data = tokio::fs::read(&file_path).await?;
        let from_host: String = self.get_host_info();
        
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
    

    #[doc = "메모리 효율코드"]
    async fn send_info_to_slave_memory(&self, file_path: &str, file_name: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error> {
        
        let from_host: String = self.get_host_info();

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
}


