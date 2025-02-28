use crate::common::*;

use crate::model::file_info::*;

use crate::configs::Configs::*;

use crate::service::file_service::*;
use crate::service::request_service::*;

#[derive(Debug, new)]
pub struct AppRouter;

impl AppRouter {
    #[doc = "Router 서비스를 등록해주는 함수"]
    /// # Arguments
    /// * `cfg` - 웹 서비스 컨피그 객체 : 새로운 서비스를 호스팅할 수 있다.
    pub fn configure_routes(cfg: &mut web::ServiceConfig) {
        cfg.service(web::resource("/upload").route(web::post().to(download_handler)));

        /* 새 라우트 추가는 아래와 같이 수행하면 된다. */
        // cfg.service(
        //     web::resource("/info")
        //         .route(web::get().to(get_info_handler))  /* 새 라우트 추가 */
        // );
    }
}

#[doc = "파일 업로드 핸들러 - master 쪽에서 수정된 파일을 넘겨주는데 해당 정보를 가지고 slave 의 파일을 최신화 해주는 함수"]
/// # Arguments
/// * `req`             - Request 객체 Http 통신을 통해서 넘어온 쿼리의 결과.
/// * `payload`         - 파일 데이터 스트림을 청크방식으로 보내줌. -> 파일 데이터.
/// * `file_service`    - file 관련 서비스 인스턴스
/// * `request_service` - request 관련 서비스 인스턴스
///
/// # Return
/// * Result<HttpResponse, Error>
async fn download_handler(
    req: web::Query<FileInfo>,
    mut payload: web::Payload,
    file_service: web::Data<Arc<FileServicePub>>,
    request_service: web::Data<Arc<RequestServicePub>>,
) -> Result<HttpResponse, Error> {
    info!("Receive a file modification signal from the master server");

    let slave_backup_path: String; /* 백업파일 경로 */
    let watch_path_string: String; /* 감시대상 파일 경로 */
    let from_host: String; /* 호스트 정보 */
    {
        let server_config: RwLockReadGuard<'_, Configs> = match get_config_read() {
            Ok(server_config) => server_config,
            Err(e) => {
                error!("[Error][upload_handler()] {:?}", e);
                return Err(actix_web::error::ErrorInternalServerError(e));
            }
        };

        watch_path_string = server_config.server.watch_path().clone();

        slave_backup_path = server_config
            .server
            .slave_backup_path()
            .clone()
            .unwrap_or(String::new());

        from_host = server_config.server.host().clone();
    }

    /* 수정된 파일의 이름 */
    let modified_file_name: String = req.filename.clone();

    /* 감시대상 파일 경로 */
    let watch_path: &Path = Path::new(watch_path_string.as_str());

    /* 수정된 파일 실제 경로 */
    let modified_file_path: PathBuf = watch_path.join(&modified_file_name);

    /*
        수정된 파일 실제 경로 문자열 변환
        - ElasticMsg 생성을 위한 변수
    */
    let modified_file_path_clone: PathBuf = modified_file_path.clone();
    let modified_file_path_str = match modified_file_path_clone.to_str() {
        Some(modified_file_path_str) => modified_file_path_str,
        None => {
            let err_msg: &str =
                "[Error][upload_handler()] Failed to convert 'modified_file_path' to string.";
            error!("{}", err_msg);
            return Err(actix_web::error::ErrorInternalServerError(err_msg));
        }
    };

    /* 파일 백업 시작 */
    let _backup_res =
        match file_service.copy_file_for_backup(modified_file_path.clone(), &slave_backup_path) {
            Ok(_) => (),
            Err(e) => {
                error!("[Error][upload_handler()] File backup Failed : {:?}", e);
                return Err(actix_web::error::ErrorInternalServerError(e));
            }
        };

    /* 전송된 파일로 기존 파일 덮어쓰기 */
    let mut chg_file: File = match File::create(modified_file_path) {
        Ok(chg_file) => chg_file,
        Err(e) => {
            error!("[Error][upload_handler()] {:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e));
        }
    };

    /* 스트림에서 데이터를 읽고 파일에 쓴다. */
    while let Ok(Some(chunk)) = payload.try_next().await {
        let data = chunk;
        let _ = chg_file.write_all(&data);
    }

    info!(
        "The file '{:?}' has been changed.",
        modified_file_path_clone
    );

    /* 아래의 코드는 해당 파일 복사 관련 로그를 Elasticsearch 에 로깅해주기 위한 코드. */
    let _es_post_res = match request_service
        .post_log_to_es(
            &from_host,
            "",
            modified_file_path_str,
            "success",
            "[slave task] Overwrite files",
        )
        .await
    {
        Ok(_) => (),
        Err(e) => {
            error!("{:?}", e);
            return Err(actix_web::error::ErrorInternalServerError(e));
        }
    };

    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}
