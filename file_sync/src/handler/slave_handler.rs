use crate::common::*;

use crate::service::file_service::*;
use crate::service::request_service::*;

use crate::middleware::middle_ware::*;

use crate::router::app_router::*;

use crate::configs::Configs::*;

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

    #[doc = "프로그램 role 이 slave 인경우의 작업"]
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let slave_host;
        let master_address;
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
        }

        let req_service = self.req_service.clone();
        let file_service = self.file_service.clone();

        HttpServer::new(move || {
            App::new()
                .wrap(CheckIp::new(master_address.clone()))
                .configure(AppRouter::configure_routes)
                .app_data(web::Data::new(file_service.clone()))
                .app_data(web::Data::new(req_service.clone()))
        })
        .bind(slave_host)?
        .run()
        .await?;

        Ok(())
    }
}
