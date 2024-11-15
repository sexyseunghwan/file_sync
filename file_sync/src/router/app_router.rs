use crate::common::*;

use crate::utils_modules::io_utils::*;

use crate::model::FileInfo::*;

use crate::service::config_request_service::*;
use crate::service::watch_service::*;

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


#[doc = "파일 업로드 핸들러 - master 쪽에서 수정된 파일을 넘겨주는데 해당 정보를 가지고 slave 의 파일을 최신화 해주는 함수"]
async fn upload_handler(
    req: web::Query<FileInfo>,
    mut payload: web::Payload,
    config_req_service: web::Data<Arc<ConfigRequestServicePub>>,
) -> Result<HttpResponse, Error> {
    
    /* 백업파일 경로 */
    let slave_backup_path = match config_req_service.get_slave_backup_path() {
        Ok(slave_backup_path) => slave_backup_path,
        Err(e) => {
            error!("{:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e.to_string()))
        }
    };

    /* 수정된 파일의 이름 */
    let file_name = req.filename.clone(); 
    
    /* 감시대상 파일 경로 */
    let watch_path_string = config_req_service.get_watch_dir_info();
    let watch_path = Path::new(watch_path_string.as_str());
    let watch_file_path: PathBuf = watch_path.join( &file_name); 

    
    /* 파일 백업 시작 */
    let _backup_res = match copy_file_for_backup(watch_file_path.clone(), &slave_backup_path) {
        Ok(_) => (),
        Err(e) => {
            error!("{:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e))
        }
    };
    
    /* 전송된 파일로 기존 파일 덮어쓰기 */
    let mut chg_file = match File::create(watch_file_path) {
        Ok(chg_file) => chg_file,
        Err(e) => {
            error!("[Error][upload()] {:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e))
        }
    };
    
    /* 스트림에서 데이터를 읽고 파일에 쓴다. */ 
    while let Ok(Some(chunk)) = payload.try_next().await {
        let data = chunk;
        let _ = chg_file.write_all(&data);
    }
    
    info!("The file '{:?}' has been changed.", watch_path);
    
    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}