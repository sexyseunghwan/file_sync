use crate::common::*;

//use crate::service::config_request_service::*;
use crate::service::request_service::*;
use crate::service::watch_service::*;

#[derive(Debug)]
pub struct MasterHandler<C,W>
where 
    C: ConfigRequestService + Sync + Send + 'static,
    W: WatchService + Sync + Send + 'static,
{
    config_req_service: Arc<C>,
    watch_service: Arc<W>
}


impl<C,W> MasterHandler<C,W> 
where
    C: ConfigRequestService + Sync + Send + 'static,
    W: WatchService + Sync + Send + 'static
{

    pub fn new(config_req_service: Arc<C>, watch_service: Arc<W>) -> Self {
        Self {
            config_req_service,
            watch_service,
        }
    }
    
    
    #[doc = "프로그램 role 이 master 인경우의 작업"]
    pub async fn run(&self) -> Result<(), anyhow::Error> {

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

                            /* 
                                변경이 감지된 파일 경로를 파싱해주는 부분
                                - windows os 의 경우 경로 앞에 의미없는 문자열이 붙는걸 확인. 해당 문자열을 제거해야함. 
                            */
                            let cleaned_path = file_path.replace(r"\\?\", "");
                            
                            tx_clone.send(Ok(cleaned_path.clone()))
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
        

        /* 
            rx 부분 - for문을 통해서 receive 를 계속 감시한다.
        */
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

                    /* 변경 파일이 있는 경우 -> slave 파일에 변경 파일을 보내준다. */
                    if watch_res {

                        match self.config_req_service.send_info_to_slave(&file_name).await {
                            Ok(_) => {
                                info!("Successfully sent files to slave servers. : {}", &file_name);
                                ()
                            },
                            Err(e) => {
                                error!("{:?}", e);
                                continue
                            }
                        }
                    }   
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


}