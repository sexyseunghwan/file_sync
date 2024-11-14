use crate::common::*;

use crate::service::config_request_service::*;
use crate::service::watch_service::*;
use crate::service::api_service::*;

use crate::middleware::middle_ware::*;



#[derive(Debug)]
pub struct MainHandler<C,W>
where 
    C: ConfigRequestService,
    W: WatchService,
{
    config_req_service: C,
    watch_service: W
}


impl<C,W> MainHandler<C,W> 
where
    C: ConfigRequestService,
    W: WatchService
{
    
    pub fn new(config_req_service: C, watch_service: W) -> Self {
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
            
            // match self.slave_task().await {
            //     Ok(_) => (),
            //     Err(e) => {
            //         error!("{:?}", e);
            //     }
            // }
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

                            /* 
                                변경이 감지된 파일 경로를 파싱해주는 부분
                                - 이쪽을 앞의 네글짜 제외하는 방식으로 하면 안될듯. 
                            */
                            //let file_path_slice = &file_path.chars().skip(4).collect::<String>();
                            let file_path_slice = &file_path.chars().collect::<String>();

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

    
    // #[doc = "프로그램 role 이 slave 인경우의 작업"]
    // pub async fn slave_task(&self) -> Result<(), anyhow::Error> {

    //     let slave_host = self.config_req_service.get_slave_host()?;
    //     let master_address = self.config_req_service.get_master_address()?;


    //     // 아 이쪽 코드 부분 상당히 마음에 안드는데 진짜로...?!
    //     let config_req_service_clone = Arc::clone(&self.config_req_service);
    //     let watch_service_clone = Arc::clone(&self.watch_service);
        
    //     HttpServer::new(move || {
    //         App::new()
    //             .wrap(CheckIp::new(master_address.clone()))
    //             .app_data(config_req_service_clone.clone())
    //             .app_data(watch_service_clone.clone())
    //             .service(upload)
    //     })
    //     .bind(slave_host)?
    //     .run()
    //     .await?;
        
    //     Ok(())
    // }
    
    
    //.route("/upload", web::post().to(self.upload()))
    // async fn upload(&self, req: web::Query<FileInfo>, mut payload: web::Payload) -> impl Responder {


    //     HttpResponse::Ok().body("File uploaded successfully")
    // }

    // #[post("/upload")]
    // async fn upload(req: web::Query<FileInfo>, mut payload: web::Payload) -> impl Responder {
        
    //     println!("Received file: {}", req.filename);
        
    //     let mut file = File::create("./download_file/uploaded_file.txt").expect("Failed to create file");

    //     /* 스트림에서 데이터를 읽고 파일에 쓴다. */ 
    //     while let Ok(Some(chunk)) = payload.try_next().await {
    //         let data = chunk;
    //         let _ = file.write_all(&data);
    //     }
            
    //     HttpResponse::Ok().body("File uploaded successfully")
    // }

}