use crate::common::*;

use crate::service::config_service::*;
use crate::service::request_service::*;

use crate::model::HashStorage::*;

use crate::utils_modules::hash_utils::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct MainHandler<C: ConfigService, R: RequestService> {
    config_service: C,
    request_service: R
}


impl<C: ConfigService, R: RequestService> MainHandler<C,R> {

    pub fn new(config_service: C, request_service: R) -> Self {
        Self { config_service, request_service }
    }
    
    #[doc = "해당 프로그램이 master role 인지 slave role 인지 정해준다."]
    pub async fn task_main(&self) {

        let role = self.config_service.get_role();
        
        if role == "master" {
            
            match self.master_task().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                }
            }

        } else {
            
            match self.slave_task().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }
    
    
    #[doc = "프로그램 role 이 master 인경우의 작업"]
    pub async fn master_task(&self) -> Result<(), anyhow::Error> {   
        
        let mut hotwatch = Hotwatch::new()?;

        /* Hash 디렉토리 */
        let hash_dir_address: Arc<String> = Arc::new(format!("{}\\hash_storage.json", self.config_service.get_watch_dir_info()));
        
        /* 감시할 파일 리스트 */
        let slave_address_vec = self.config_service.get_watch_file_list();
        
        /* 해당 파일을 계속 감시해준다. */
        let (tx, rx) = channel::<Result<(), String>>();
        
        
        for file in slave_address_vec.iter() {
         
            let file_path = file.to_string();
            let hash_dir_address_clone = Arc::clone(&hash_dir_address);
            
            let tx_clone = tx.clone();
            
            /* hotwatch event 를 관리해준다. */
            hotwatch.watch(file_path, move |event: Event| {
                
                if let WatchEventKind::Modify(_) = event.kind {
                    
                    println!("{:?} changed!", event.paths[0]);
                    
                    /* 1안 */
                    match event.paths[0].to_str() {
                        
                        Some(file_path) => {

                            /* 변경이 감지된 파일 경로를 파싱해주는 부분 */
                            let file_path_slice = &file_path.chars().skip(4).collect::<String>();
                            
                            match HashStorage::load(Path::new(&*hash_dir_address_clone)) {
                                Ok(hash_storage) => {
                                    /* hash storage 를 얻었으므로 여기서 비교를 해준다. */

                                    /* 현재 이벤트가 걸린 파일의 해쉬값 */
                                    let event_file_hash_val: Vec<u8> = conpute_hash(Path::new(file_path_slice))
                                        .unwrap_or_else(|_| vec![]);
                                    
                                    /* 기존 해당 파일 해쉬값 */
                                    let in_storage_file_hash_val = hash_storage.get_hash(file_path_slice);
                                    
                                    if in_storage_file_hash_val != event_file_hash_val {
                                        
                                    }
                                    
                                },
                                Err(e) => {
                                    tx_clone.send(Err(format!("Failed to load hash storage: {}", e)))
                                        .unwrap_or_else(|err| error!("Failed to send error message: {}", err));
                                }
                            } 
                        },
                        None => {
                            tx_clone.send(Err("Failed to detect file path".to_string()))
                                .unwrap_or_else(|err| error!("Failed to send error message: {}", err));
                        }
                    }
                    
                    

                    /* 2안 */
                    // if let Some(file_path) =  event.paths[0].to_str() {

                    //     /* 변경이 감지된 파일 경로를 파싱해주는 부분 */
                    //     let file_path_slice = &file_path.chars().skip(4).collect::<String>();
                        
                    //     let mut hash_storage = match HashStorage::load(Path::new(&*hash_dir_address_clone)) {
                    //         Ok(hash_storage) => hash_storage,
                    //         Err(_e) => {
                    //             tx_clone.send(Err("Failed to detect file path".to_string())).expect("test")
                    //         }
                    //     };
                        


                    //     //anyhow!("error");
                    //     tx_clone.send(Err("Failed to detect file path".to_string())).expect("test");
                        
                    //     println!("how how how");

                    // } else {
                    //    //anyhow!("error");
                    //    tx_clone.send(Err("Failed to detect file path".to_string())).expect("test1")
                    // };
                    
                    println!("???");
                }
            })?
        }
        
        /* for문을 통해서 receive 를 계속 감시한다. */
        for received in rx {
            match received {
                Ok(_) => {
                    info!("File changed successfully.")
                },
                Err(e) => {
                    error!("Error processing file change: {:?}", e);
                    /* 에러 발생시 다음 수신으로 전환 */
                    continue; 
                }
            }
        }
        
        Ok(())
        
    }   
    

            /* loop를 통해서 계속 감시 진행. */
        // loop {
            
        //     match rx.recv() {
        //         Ok(_) => info!("Received file change event."),
        //         Err(e) => error!("Error receiving from channel: {}", e),
        //     }
        // }
        

    #[doc = "프로그램 role 이 slave 인경우의 작업"]
    pub async fn slave_task(&self) -> Result<(), anyhow::Error> {

        

        Ok(())
    }

}