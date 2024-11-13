use crate::common::*;

use crate::service::config_request_service::*;
use crate::service::watch_service::*;


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
        let (tx, rx) = channel::<Result<String, String>>();
        
        /* tx 부분 */
        for monitor_file in &slave_address_vec {
            
            let file_path = monitor_file.to_string();
            let tx_clone = tx.clone();  /* tx의 복사본을 생성 -> 소유권 때문 위의 for 문 때문임. */
            
            hotwatch.watch(file_path.clone(), move |event: Event| {

                if let WatchEventKind::Modify(_) = event.kind {
                    
                    match event.paths[0].to_str() {
                        Some(file_path) => {

                            /* 변경이 감지된 파일 경로를 파싱해주는 부분 */
                            let file_path_slice = &file_path.chars().skip(4).collect::<String>();
                            
                            tx_clone.send(Ok(file_path_slice.clone()))
                                .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));
                        },
                        None => {
                            tx_clone.send(Err("[Error][master_task()] Failed to detect file path".to_string()))
                                .unwrap_or_else(|err| error!("[Error][master_task()] Failed to send error message: {}", err));
                        }
                    }
                }
            })?;
        }
        
        
        /* rx 부분 */
        /* for문을 통해서 receive 를 계속 감시한다. */
        for received in rx {

            match received {
                Ok(file_name) => {
                    
                    let watch_res = match self.watch_service.comparison_file(&file_name) {
                        Ok(watch_res) => watch_res,
                        Err(e) => {
                            error!("{:?}", e);
                            continue
                        }
                    };

                    // /* 변경 파일이 있는 경우 -> slave 파일에 변경 파일을 보내준다. */
                    // if watch_res {
                    //     match self.config_req_service.send_info_to_slave(&file_name).await {
                    //         Ok(_) => (),
                    //         Err(e) => {
                    //             error!("{:?}", e);
                    //             continue
                    //         }
                    //     }
                    // }   
                    
                    info!("File changed successfully.: {}", &file_name);
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
    


    
    #[doc = "프로그램 role 이 slave 인경우의 작업"]
    pub async fn slave_task(&self) -> Result<(), anyhow::Error> {
        
        

        Ok(())
    }

}