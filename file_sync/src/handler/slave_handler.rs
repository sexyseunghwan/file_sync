use crate::common::*;

use crate::traits::service::{file_service::*, request_service::*};

use crate::middleware::middle_ware::*;

use crate::router::app_router::*;

use crate::configs::configs::*;

use crate::utils_modules::tls_utils::*;

#[derive(Debug)]
pub struct SlaveHandler<R, F>
where
    R: RequestService + Sync + Send + 'static,
    F: FileService + Sync + Send + 'static,
{
    req_service: Arc<R>,
    file_service: Arc<F>,
}

impl<R, F> SlaveHandler<R, F>
where
    R: RequestService + Sync + Send + 'static,
    F: FileService + Sync + Send + 'static,
{
    pub fn new(req_service: Arc<R>, file_service: Arc<F>) -> Self {
        Self {
            req_service,
            file_service,
        }
    }
    
    #[doc = "프로그램 role 이 slave 인경우의 작업: 보안 모드에 따라 HTTP 또는 mTLS HTTPS 서버를 실행한다."]
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let slave_host: String;
        let master_address: Vec<String>;
        let secure_mode: bool;
        {
            let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;
            slave_host = server_config.server.host().to_string();
            master_address = server_config
                .server
                .master_address()
                .as_ref()
                .ok_or_else(|| {
                    anyhow!("[Error][run()] The information 'master_address' does not exist.")
                })?
                .clone();
            secure_mode = server_config.server.is_secure_mode();
        }

        let file_service: Arc<F> = self.file_service.clone();

        /* TLS 를 적용한 경우 */
        if secure_mode {
            let tls_config: rustls::ServerConfig = create_server_tls_config()
                .map_err(|e| anyhow!("[ERROR][SlaveHandler->run] Failed to create TLS config: {}", e))?;

            info!("Starting secure slave server with mTLS on: {}", slave_host);

            HttpServer::new(move || {
                App::new()
                    .wrap(CheckIp::new(master_address.clone()))
                    .configure(AppRouter::configure_routes)
                    .app_data(web::Data::new(file_service.clone()))
            })
            .bind_rustls_0_23(&slave_host, tls_config)?
            .run()
            .await?;
        } else {
            /* TLS 를 적용하지 않은 경우 */
            info!("Starting regular HTTP slave server on: {}", slave_host);

            HttpServer::new(move || {
                App::new()
                    .wrap(CheckIp::new(master_address.clone()))
                    .configure(AppRouter::configure_routes)
                    .app_data(web::Data::new(file_service.clone()))
            })
            .bind(slave_host)?
            .run()
            .await?;
        }

        Ok(())
    }
}
