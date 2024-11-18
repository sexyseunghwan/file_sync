use crate::common::*;

use crate::router::app_router;
//use crate::service::config_request_service::*;
use crate::service::request_service::*;
use crate::service::watch_service::*;

use crate::middleware::middle_ware::*;

use crate::router::app_router::*;

#[derive(Debug)]
pub struct SlaveHandler<C,W>
where 
    C: ConfigRequestService + Sync + Send + 'static,
    W: WatchService + Sync + Send + 'static,
{
    config_req_service: Arc<C>,
    watch_service: Arc<W>
}


impl<C,W> SlaveHandler<C,W> 
where
    C: ConfigRequestService + Sync + Send + 'static,
    W: WatchService + Sync + Send + 'static
{
    
    pub fn new(config_req_service: Arc<C>, watch_service: Arc<W>) -> Self {
        Self {
            config_req_service,
            watch_service,
        }
    }
    
    
    #[doc = "프로그램 role 이 slave 인경우의 작업"]
    pub async fn run(&self) -> Result<(), anyhow::Error> {

        let config_req_service = self.config_req_service.clone();
        let watch_service = self.watch_service.clone();

        let slave_host = self.config_req_service.get_host_info();
        let master_address = self.config_req_service.get_master_address()?;
        
        HttpServer::new(move || {
            App::new()
                .wrap(CheckIp::new(master_address.clone()))
                .configure(AppRouter::configure_routes)
                .app_data(web::Data::new(config_req_service.clone()))
                .app_data(web::Data::new(watch_service.clone()))
        })
        .bind(slave_host)?
        .run()
        .await?;
        
        Ok(())
    }


}