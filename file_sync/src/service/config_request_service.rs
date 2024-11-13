use anyhow::anyhow;

use crate::common::*;

use crate::model::Configs::*;

use crate::utils_modules::io_utils::*;


#[async_trait]
pub trait ConfigRequestService {
    fn get_role(&self) -> String;
    fn get_slave_address(&self) -> Result<Vec<String>, anyhow::Error>;
    fn get_watch_file_list(&self) -> Vec<String>;
    fn get_watch_dir_info(&self) -> String;
    

    //async fn handle_file_change(&self, file_path: &str, tx: &Sender<Result<(), String>>) -> Result<(), anyhow::Error>;
    //fn watch_file(&self) -> Result<(Result<(), SendError<()>>, Receiver<()>), anyhow::Error>;
    //fn get_watcher_delegator() -> Result<>;
    
    async fn send_info_to_slave(&self, file_path: &str) -> Result<(), anyhow::Error>;
    async fn send_info_to_slave_io(&self, file_path: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error>;
    async fn send_info_to_slave_memory(&self, file_path: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error>;
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
    
    #[doc = ""]
    async fn send_info_to_slave(&self, file_path: &str) -> Result<(), anyhow::Error> {

        let slave_url = self
            .config
            .server
            .slave_address
            .as_ref()
            .ok_or_else(|| anyhow!("[Error][send_info_to_slave()] 'slave_url' not found."))?;
           
        let io_improvement_option = self.config.server.io_bound_improvement;
        
        if io_improvement_option {
            self.send_info_to_slave_io(file_path, slave_url.clone()).await?;
        } else {
            self.send_info_to_slave_memory(file_path, slave_url.clone()).await?;
        }

        Ok(())
    }
    
    
    #[doc = "i/o 바운드 효율코드"]
    async fn send_info_to_slave_io(&self, file_path: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error> {

        let file_data = tokio::fs::read(file_path).await?;
        
        let tasks: Vec<_> = slave_url.into_iter().map(|url| {
            
            let client = self.client.clone();
            let data_clone = file_data.clone();
            let parsing_url = format!("http://{}/upload",url);
            
            task::spawn(async move {
                
                let body = Body::from(data_clone);
                let response = client.post(&parsing_url)
                    .body(body)
                    .send()
                    .await;
    
                match response {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            info!("File was sent successfully: {}", &url);
                            Ok(())
                        } else {
                            Err(anyhow::anyhow!("[Error][send_info_to_slave_io()] Failed to send file: {}", resp.status()))
                        }
                    },
                    Err(e) => Err(anyhow::anyhow!("[Error][send_info_to_slave_io()] HTTP request failed: {}", e)),
                }
            })
        }).collect();

        let results = join_all(tasks).await;
        results.into_iter().collect::<Result<Vec<_>, _>>()?;
        
        Ok(())
    }
    

    #[doc = "메모리 효율코드"]
    async fn send_info_to_slave_memory(&self, file_path: &str, slave_url: Vec<String>) -> Result<(), anyhow::Error> {
        
        let tasks: Vec<_> = slave_url.into_iter().map(|url| {
                
            let client = self.client.clone();
            let parsing_url = format!("http://{}/upload",url);
            let file_path = file_path.to_string().clone();

            task::spawn(async move {
                
                let file_data = tokio::fs::read(file_path).await?;
                let body = Body::from(file_data);
                let response = client.post(&parsing_url)
                    .body(body)
                    .send()
                    .await;
    
                match response {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            info!("File was sent successfully: {}", &url);
                            Ok(())
                        } else {
                            Err(anyhow::anyhow!("[Error][send_info_to_slave_io()] Failed to send file: {}", resp.status()))
                        }
                    },
                    Err(e) => Err(anyhow::anyhow!("[Error][send_info_to_slave_io()] HTTP request failed: {}", e)),
                }
            })
        }).collect();

        let results = join_all(tasks).await;
        results.into_iter().collect::<Result<Vec<_>, _>>()?;
        
        Ok(())
    }


    
        // for url in slave_url {

        //     let file_data = tokio::fs::read(file_path).await?;

        //     let data_clone = file_data.clone();
        //     let body = Body::from(data_clone);
            
        //     let response = self.client.post(&url)
        //         .body(body)
        //         .send()
        //         .await?;

        //     if response.status().is_success() {
        //         info!("File was sent successfully.: {}", &url);
        //     } else {
        //         anyhow!("[Error][send_info_to_slave_memory()] Failed to send file: {}", response.status());
        //     }
        // }

    // #[doc = ""]
    // async fn handle_file_change(&self, file_path: &str, tx: &Sender<Result<(), String>>) -> Result<(), anyhow::Error> {

        
        

    //     Ok(())
    // }

    // fn watch_file(&self) -> Result<(Result<(), SendError<()>>, Receiver<()>), anyhow::Error> {

    //     /* file 리스트를 가져온다. */
    //     let files = self.config.server.specific_files.clone();
        
    //     /* 지정된 file 리스트가 하나도 없다면 */
    //     if files.len() == 0 {
    //         return Err(anyhow!("[Error][watch_file()] There is no list of files to monitor."))
    //     }
        
        
    //     let mut hotwatch = Hotwatch::new()?;

    //     let (tx, rx) = channel();

    //     let value: Result<(), SendError<()>> = tx.send(());

    //     for file in files.iter() {
         
    //         let file_path = file.to_string();
            
    //         hotwatch.watch(file_path, move |event: Event| {
            
    //             if let WatchEventKind::Modify(_) = event.kind {
    //                 value.expect("Failed to send event");
    //                 //println!("{:?} changed!", event.paths[0]);
    //             }
        
    //         })?
    //     }

    //     Ok((value, rx))
    // }

    // #[doc = "docs"]
    // fn get_watcher_delegator() -> Result<> {
        


    // }

}