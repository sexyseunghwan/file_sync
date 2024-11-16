mod common;

use crate::common::*;

mod utils_modules;
use controller::main_controller;
use utils_modules::logger_utils::*;

mod handler;
use handler::main_handler::*;

mod model;

mod service;
use service::config_request_service::*;
use service::watch_service::*;

mod repository;

mod middleware;

mod router;

mod controller;
use controller::main_controller::*;


#[tokio::main]
async fn main() {

    /* 로깅 시작 */
    set_global_logger();
    info!("File Sync Program Start");
    
    /* 종속 서비스 호출 */
    let config_req_service = Arc::new(ConfigRequestServicePub::new());
    let watch_service = Arc::new(WatchServicePub::new());   
    
    /* 메인 컨트롤러 호출 */
    let main_controller = MainController::new(config_req_service, watch_service);

    /* 메인함수 호출 */
    main_controller.task_main().await;

    

    /* 메인핸들러 호출 */
    //let main_handler = MainHandler::new(config_req_service, watch_service);
    /* 메인 함수 */
    //main_handler.task_main().await;
}