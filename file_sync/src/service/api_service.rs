use crate::common::*;

use crate::model::FileInfo::*;

use crate::service::config_request_service::*;
use crate::service::watch_service::*;



#[post("/upload")]
async fn upload
(
    req: web::Query<FileInfo>, 
    mut payload: web::Payload,
    config_req_service: web::Data<dyn ConfigRequestService>,
    watch_service: web::Data<dyn WatchService>
    
) -> Result<HttpResponse, Error> {
    
    let monitoring_path = config_req_service.get_watch_dir_info();
    //let chg_file_dir = format!("{}\\{}", monitoring_path, req.filename);
    let chg_file_dir = format!("{}/{}", monitoring_path, req.filename);

    let mut chg_file = match File::create(chg_file_dir) {
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
     
    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}
    

