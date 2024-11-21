
/*
Author      : Seunghwan Shin 
Create date : 2024-11-20 
Description : Elasticsearch 중앙 집중식 사전 관리 시스템
    
History     : 2024-11-20 Seunghwan Shin       # [v.1.0.0] first create
*/

mod common;
use crate::common::*;

mod utils_modules;
use utils_modules::logger_utils::*;

mod handler;

mod model;

mod service;
use service::request_service::*;
use service::file_service::*;

mod repository;

mod middleware;

mod router;

mod controller;
use controller::main_controller::*;

mod configs;


#[tokio::main]
async fn main() {
    
    /* 로깅 시작 */
    set_global_logger();
    info!("File Sync Program Start");
    
    /* 종속 서비스 호출 */
    let config_req_service = Arc::new(RequestServicePub::new());
    let file_service = Arc::new(FileServicePub::new());   
    
    /* 메인 컨트롤러 호출 */
    let main_controller = MainController::new(config_req_service, file_service);

    /* 메인함수 호출 */
    main_controller.task_main().await;
    
    


    /* ==== TEST 시나리오 ==== */
    //let mut handles = vec![];
    
    // use repository::request_repository::*;

    // let mut handles: Vec<task::JoinHandle<()>> = Vec::new();
    
    // for i in 0..10 {
        
    //     let client_clone = get_request_client();

    //     let handle = tokio::spawn(async move {
            
    //         let url = format!("http://192.168.0.98:9000/test?filename={}", i);

    //         let test = client_clone.post(&url).send().await.unwrap();

    //         tokio::time::sleep(Duration::from_secs(2)).await;
    //         println!("Task {} completed.", i);
    //     });
    //     handles.push(handle);
    // }

    // // 모든 태스크가 완료될 때까지 기다림
    // for handle in handles {
    //     handle.await.unwrap();
    // }
    

    // for i in 0..5 {
        
    //     //let client_clone = Arc::clone(&client_arc); // 각 스레드에서 사용할 Client의 클론 생성
    //     let client_clone = get_request_client();


    //     let handle = tokio::spawn(move || {
    //         // 각 스레드에서 HTTP 요청 수행
    //         let url = format!("http://192.168.0.98/upload?filename={}", i);
            
    //         let test = client_clone.client.post(&url).send()
            
    //         match client_clone.client.post(&url).send() {
    //             Ok(response) => {
    //                 println!("Thread {}: Received response: {}", i, response.status());
    //             }
    //             Err(e) => {
    //                 println!("Thread {}: Error sending request: {}", i, e);
    //             }
    //         }
    //     });

    //     handles.push(handle);
    //     std::thread::sleep(Duration::from_millis(100)); // 간단한 딜레이
    // }

    // // 모든 스레드가 완료되기를 기다림
    // for handle in handles {
    //     handle.join().unwrap();
    // }
}