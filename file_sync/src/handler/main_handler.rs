use crate::common::*;

use crate::service::config_service::*;
use crate::service::request_service::*;


use crate::utils_modules::hash_utils::*;

use crate::repository::hash_repository::*;


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
        
        /* Config file 의 role 옵션에 따라서 결정됨. */
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
        
        /* 감시할 파일 리스트 */
        let slave_address_vec = self.config_service.get_watch_file_list();
        
        /* 해당 파일을 계속 감시해준다. */
        //let (tx, rx) = channel::<Result<(), String>>();
        let (tx, rx) = channel::<Result<(), String>>();
        
        for monitor_file in &slave_address_vec {

            let file_path = monitor_file.to_string();
            
            hotwatch.watch(file_path, move |event: Event| { 
                

                if let WatchEventKind::Modify(_) = event.kind {

                    match event.paths[0].to_str() {
                        Some(file_path) => {
                            
                            /* 변경이 감지된 파일 경로를 파싱해주는 부분 */
                            let file_path_slice = &file_path.chars().skip(4).collect::<String>();


                        }
                        None => {
                            tx.send(Err("Failed to detect file path".to_string()))
                                .unwrap_or_else(|err| error!("Failed to send error message: {}", err));
                        }
                    }
                }

            });

        }
        
        //let tx_clone: Sender<Result<(), String>> = tx.clone();

        // for monitor_file in &slave_address_vec {
            
        //     let file_path = monitor_file.to_string();
            
        //     hotwatch.watch(file_path.clone(), move |event| {
                
        //         let tx = tx.clone();
                
        //         tokio::spawn(async move {
                    
        //             if let WatchEventKind::Modify(_) = event.kind {
                        
        //                 if let Some(file_path_str) = event.paths.get(0).and_then(|path| path.to_str()) {
        //                     self.config_service.handle_file_change(file_path_str, &tx).await;
        //                 } else {
        //                     let err_msg = "Failed to detect file path";
        //                     error!("{}", err_msg);
        //                     tx.send(Err(err_msg.to_string()))
        //                         .unwrap_or_else(|err| error!("Failed to send error message: {}", err));
        //                 }
        //             }
        //         });
        //     })?;
        // }
        
        /* 보류 */
        for monitor_file in slave_address_vec.iter() {
            
            /* 감시대상 파일 경로 */
            let file_path = monitor_file.to_string();       
            let tx_clone = tx.clone();
            
            /* hotwatch event 를 관리해준다. */
            hotwatch.watch(file_path, move |event: Event| {
                
                if let WatchEventKind::Modify(_) = event.kind {
                    
                    /* 1안 */
                    match event.paths[0].to_str() {
                        
                        Some(file_path) => {
                            
                            /* 변경이 감지된 파일 경로를 파싱해주는 부분 */
                            let file_path_slice = &file_path.chars().skip(4).collect::<String>();
                            
                            /*  
                                현재 이벤트가 걸린 파일의 Hash value 
                                - 문제가 발생할 경우 empty vector 반환    
                            */  
                            let event_hash: Vec<u8> = conpute_hash(Path::new(file_path_slice))
                                .unwrap_or_else(|_| vec![]);
                            
                            let storage_hash_guard = get_hash_storage();
                            let storage_hash = storage_hash_guard
                                .read()
                                .unwrap()
                                .get_hash(file_path_slice);
                            
                            if storage_hash != event_hash {
                                
                                let storage_hash_write_guard = get_hash_storage();
                                let mut storage_hash_write = storage_hash_write_guard
                                    .write()
                                    .unwrap();
                                
                                storage_hash_write.update_hash(file_path_slice.clone(), storage_hash);
                                storage_hash_write.save().unwrap()
                            }
                        },
                        None => {
                            tx_clone.send(Err("Failed to detect file path".to_string()))
                                .unwrap_or_else(|err| error!("Failed to send error message: {}", err));
                        }
                    }
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
        
        // while let Some(result) = rx.recv().await {
        //     match result {
        //         Ok(_) => info!("File changed successfully."),
        //         Err(e) => error!("Error processing file change: {:?}", e),
        //     }
        // }
        

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