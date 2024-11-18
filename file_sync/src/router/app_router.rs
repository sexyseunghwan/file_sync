use crate::common::*;

use crate::model::ElasticMsg::*;

use crate::utils_modules::io_utils::*;
use crate::utils_modules::request_utils::*;

use crate::model::FileInfo::*;

use crate::configs::Configs::*;

use crate::service::request_service::*;

#[derive(Debug, new)]
pub struct AppRouter;

impl AppRouter {
    
    #[doc = ""]
    pub fn configure_routes(cfg: &mut web::ServiceConfig) {
        
        cfg.service(
            web::resource("/upload")
                .route(web::post().to(upload_handler))
        );
        
        /* 새 라우트 추가는 아래와 같이 수행하면 된다. */
        // cfg.service(
        //     web::resource("/info")
        //         .route(web::get().to(get_info_handler))  /* 새 라우트 추가 */ 
        // );
    }
}


#[doc = ""]
async fn router_log_es_posting(from_host: &str, to_host: &str, file_name: &str, task_status: &str, task_detail: &str) -> Result<(), anyhow::Error> {

    let es_msg = ElasticMsg::new(
        from_host, 
        to_host, 
        file_name, 
        task_status, 
        task_detail)?;
    
    send_task_message_to_elastic(es_msg).await?;

    Ok(())
}


#[doc = "파일 업로드 핸들러 - master 쪽에서 수정된 파일을 넘겨주는데 해당 정보를 가지고 slave 의 파일을 최신화 해주는 함수"]
async fn upload_handler(
    req: web::Query<FileInfo>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    
    info!("Receive a file modification signal from the master server");
    
    let slave_backup_path;
    {
        let server_config = match get_config_read() {
            Ok(server_config) => server_config,
            Err(e) => {
                return Err(actix_web::error::ErrorInternalServerError(e.to_string()))
            }
        };

        let test = match server_config
            .server
            .slave_backup_path() {
                Some(test) => test,
                None
            } 
    }

    /* 백업파일 경로 */
    let slave_backup_path = match req_service.get_slave_backup_path() {
        Ok(slave_backup_path) => slave_backup_path,
        Err(e) => {
            error!("[Error][upload_handler()] {:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e.to_string()))
        }
    };

    /* 수정된 파일의 이름 */
    let file_name = req.filename.clone(); 
    
    /* 감시대상 파일 경로 */
    let watch_path_string = req_service.get_watch_dir_info();
    let watch_path = Path::new(watch_path_string.as_str());
    let watch_file_path: PathBuf = watch_path.join( &file_name); 
    
    /* 파일 백업 시작 */
    let _backup_res = match copy_file_for_backup(watch_file_path.clone(), &slave_backup_path) {
        Ok(_) => (),
        Err(e) => {
            error!("[Error][upload_handler()] {:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e))
        }
    };
    
    /* 전송된 파일로 기존 파일 덮어쓰기 */
    let mut chg_file = match File::create(watch_file_path) {
        Ok(chg_file) => chg_file,
        Err(e) => {
            error!("[Error][upload_handler()] {:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e))
        }
    };
    
    /* 스트림에서 데이터를 읽고 파일에 쓴다. */ 
    while let Ok(Some(chunk)) = payload.try_next().await {
        let data = chunk;
        let _ = chg_file.write_all(&data);
    }
    
    info!("The file '{:?}' has been changed.", watch_path);

    /* 아래의 코드는 해당 파일 복사 관련 로그를 Elasticsearch 에 로깅해주기 위한 코드이다. */
    let from_host = req_service.get_host_info();
    
    let _es_post_res = match router_log_es_posting(
        &from_host, 
        "", 
        &file_name, 
        "success", 
        "slave task" ).await {
            Ok(_) => (),
            Err(e) => {
                error!("{:?}", e);
                return Err(actix_web::error::ErrorInternalServerError(e))
            }
    };
    
    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}



