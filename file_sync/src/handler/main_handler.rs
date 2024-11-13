use actix_web::error;

use crate::common::*;

use crate::service::config_request_service::*;
use crate::service::watch_service::*;



// #[derive(Debug)]
// pub struct MainHandler<C: ConfigService + Sync + Send + 'static, R: RequestService + Sync + Send + 'static, W: WatchService + Sync + Send + 'static> {
//     config_service: Arc<C>,
//     request_service: Arc<R>,
//     watch_service: Arc<W>
// }

#[derive(Debug)]
pub struct MainHandler<C,W>
where 
    C: ConfigRequestService + Sync + Send + 'static,
    W: WatchService + Sync + Send + 'static
{
    config_req_service: Arc<C>,
    watch_service: Arc<W>
}


impl<C,W> MainHandler<C,W> 
where
    C: ConfigRequestService + Sync + Send + 'static,
    W: WatchService + Sync + Send + 'static 
{
    
    pub fn new(config_req_service: Arc<C>, watch_service: Arc<W>) -> Self {
        Self { config_req_service, watch_service }
    }
    
    #[doc = "해당 프로그램이 master role 인지 slave role 인지 정해준다."]
    pub async fn task_main(&self) {

        let role = self.config_req_service.get_role();

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
        let slave_address_vec = self.config_req_service.get_watch_file_list();
        
        /* 해당 파일을 계속 감시해준다. */
        //let (tx, rx) = channel::<Result<(), String>>();
        let (tx, rx) = channel::<Result<String, String>>();
        
        for monitor_file in &slave_address_vec {
            
            let file_path = monitor_file.to_string();
            let tx_clone = tx.clone();  /* tx의 복사본을 생성 -> 소유권 때문 위의 for 문 때문임. */
            
             /* 
                아래의 Service 들은 static 으로 관리됨.
                - 시스템 시작과 함께 한번만 선언되기 때문에 메모리낭비는 발생하지 않음. 
            */
            //let watch_service_clone = Arc::clone(&self.watch_service);  
            //let config_req_service_clone = Arc::clone(&self.config_req_service);
            
            hotwatch.watch(file_path.clone(), move |event: Event| {

                if let WatchEventKind::Modify(_) = event.kind {

                    match event.paths[0].to_str() {
                        Some(file_path) => {
                            
                            /* 변경이 감지된 파일 경로를 파싱해주는 부분 */
                            let file_path_slice = &file_path.chars().skip(4).collect::<String>();
                            
                            tx_clone.send(Ok(file_path_slice.clone()))
                                .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));
                            //let watch_res = watch_service.monitor_file(file_path_slice); 
                            
                            /* 변경사항 체크중 에러가 발생한 경우 */
                            // if watch_res.err_flag {
                            
                            //     tx_clone.send(Err(watch_res.err_msg))
                            //         .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));
                            
                            // } else {
                                
                            //     /* 파일의 변경이 발생한 경우 */
                            //     if watch_res.change_flag {

                            //         println!("수정발생 수정발생!!!");

                            //         /* request 를 통해서 파일의 변경을 알려준다. */
                            //         // let req_msg = config_req_service.send_info_to_slave(file_path_slice).await;
                                        
                            //         // /* request 도중에 에러가 발생한 경우. */
                            //         // if req_msg.err_flag {
                            //         //     tx_clone.send(Err(req_msg.err_msg))
                            //         //         .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));
                            //         // }
                            //     }
                            // }
                        },
                        None => {
                            tx_clone.send(Err("[Error][master_task()] Failed to detect file path".to_string()))
                                .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));
                        }
                    }
                    
                    //let file_path_slice = file_path.clone();
                    // let tx_clone = tx_clone.clone();
                    // let watch_service = watch_service_clone.clone();
                    // let config_req_service = config_req_service_clone.clone();
                    //println!("test");
                    // spawn(async move {
                    
                    //     match event.paths[0].to_str() {
                    //         Some(file_path) => {
                                
                    //             /* 변경이 감지된 파일 경로를 파싱해주는 부분 */
                    //             let file_path_slice = &file_path.chars().skip(4).collect::<String>();

                    //             let watch_res = watch_service.monitor_file(file_path_slice); 

                    //             /* 변경사항 체크중 에러가 발생한 경우 */
                    //             if watch_res.err_flag {

                    //                 tx_clone.send(Err(watch_res.err_msg))
                    //                     .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));

                    //             } else {
                                    
                    //                 /* 파일의 변경이 발생한 경우 */
                    //                 if watch_res.change_flag {

                    //                     println!("수정발생 수정발생!!!");

                    //                     /* request 를 통해서 파일의 변경을 알려준다. */
                    //                     // let req_msg = config_req_service.send_info_to_slave(file_path_slice).await;
                                            
                    //                     // /* request 도중에 에러가 발생한 경우. */
                    //                     // if req_msg.err_flag {
                    //                     //     tx_clone.send(Err(req_msg.err_msg))
                    //                     //         .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));
                    //                     // }
                    //                 }
                    //             }
                    //         },
                    //         None => {
                    //             tx_clone.send(Err("[Error][master_task()] Failed to detect file path".to_string()))
                    //                 .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));
                    //         }
                    //     }

                    //     //let watch_res = watch_service_clone.monitor_file(&file_path_slice);
                        
                    //     // let watch_res = watch_service.monitor_file(&file_path_slice);
                    //     // if let Err(e) = watch_res {
                    //     //     let _ = tx_clone.send(Err(e.to_string()));
                    //     // } else if watch_res?.change_flag {
                    //     //     let req_res = config_req_service.send_info_to_slave(&file_path_slice).await;
                    //     //     if let Err(e) = req_res {
                    //     //         let _ = tx_clone.send(Err(e.to_string()));
                    //     //     }
                    //     // }
                    // });
                }
            })?;
        
            
            // let _ = hotwatch.watch(file_path, move |event: Event| { 
                
            //     if let WatchEventKind::Modify(_) = event.kind {
                    
            //         match event.paths[0].to_str() {
            //             Some(file_path) => {
                            
            //                 /* 변경이 감지된 파일 경로를 파싱해주는 부분 */
            //                 let file_path_slice = &file_path.chars().skip(4).collect::<String>();
            //                 println!("{:?}", file_path_slice);
                            
            //                 let watch_res = watch_service_clone.monitor_file(file_path_slice);  

            //                 /* 변경사항 체크중 에러가 발생한 경우 */
            //                 if watch_res.err_flag {

            //                     tx_clone.send(Err(watch_res.err_msg))
            //                         .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));

            //                 } else {

            //                     /* 파일의 변경이 발생한 경우 */
            //                     if watch_res.change_flag {
                                    
            //                         /* request 를 통해서 파일의 변경을 알려준다. */
            //                         let req_msg = config_req_service_clone.send_info_to_slave(file_path_slice).await;
                                        
            //                         // /* request 도중에 에러가 발생한 경우. */
            //                         if req_msg.err_flag {
            //                             tx_clone.send(Err(req_msg.err_msg))
            //                                 .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));
            //                         }
            //                         // println!("let it be");

                                    
            //                     }
            //                 }
            //             }
            //             None => {
            //                 tx_clone.send(Err("[Error][master_task()] Failed to detect file path".to_string()))
            //                     .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));
            //             }
            //         }
            //     }
            // });
        }


        /* 
            아래의 Service 들은 static 으로 관리됨.
            - 시스템 시작과 함께 한번만 선언되기 때문에 메모리낭비는 발생하지 않음. 
        */
        let watch_service_clone = Arc::clone(&self.watch_service);  
        let config_req_service_clone = Arc::clone(&self.config_req_service);   
        
        /* for문을 통해서 receive 를 계속 감시한다. */
        for received in rx {

            println!("?!");
            match received {
                Ok(file_name) => {
                    
                    //let watch_res = watch_service_clone.monitor_file(&file_name);
                    
                    // let test = match watch_service_clone.test().await {
                    //     Ok(_) => () ,
                    //     Err(e) => {
                    //         error!("{:?}", e);
                    //         continue
                    //     }
                    // };
                    
                    info!("File changed successfully.");
                },
                Err(e) => {
                    error!("[Error][master_task()] Error processing file change: {:?}", e);
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