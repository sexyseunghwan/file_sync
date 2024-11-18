use crate::common::*;

use crate::router::app_router;
use crate::service::request_service::*;
use crate::service::watch_service::*;

use crate::middleware::middle_ware::*;

use crate::router::app_router::*;

use crate::configs::Configs::*;

#[derive(Debug)]
pub struct SlaveHandler<R,W>
where 
    R: RequestService + Sync + Send + 'static,
    W: WatchService + Sync + Send + 'static,
{
    req_service: Arc<R>,
    watch_service: Arc<W>
}


impl<R,W> SlaveHandler<R,W> 
where
    R: RequestService + Sync + Send + 'static,
    W: WatchService + Sync + Send + 'static
{
    
    pub fn new(req_service: Arc<R>, watch_service: Arc<W>) -> Self {
        Self {
            req_service,
            watch_service,
        }
    }
    
    
    #[doc = "프로그램 role 이 slave 인경우의 작업"]
    pub async fn run(&self) -> Result<(), anyhow::Error> {

        let config_req_service = self.req_service.clone();
        let watch_service = self.watch_service.clone();


        let slave_host;
        let master_address;
        {
            let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;
            slave_host = server_config.server.host().to_string();
            master_address = server_config
                .server
                .master_address()
                .as_ref()
                .ok_or_else(|| anyhow!("[Error][run()] The information 'master_address' does not exist."))?
                .clone();
        }
        

        HttpServer::new(move || {
            App::new()
                .wrap(CheckIp::new(master_address.clone()))
                .configure(AppRouter::configure_routes)
                //.app_data(web::Data::new(config_req_service.clone()))
                //.app_data(web::Data::new(watch_service.clone()))
        })
        .bind(slave_host)?
        .run()
        .await?;
        
        Ok(())
    }


}