
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
}