use crate::common::*;

use crate::service::config_request_service::*;
use crate::service::watch_service::*;

use crate::handler::master_handler::*;
use crate::handler::slave_handler::*;


#[derive(Debug)]
pub struct MainController<C, W>
where
    C: ConfigRequestService + Sync + Send + 'static,
    W: WatchService + Sync + Send + 'static,
{
    config_req_service: Arc<C>,
    watch_service: Arc<W>,
}


impl<C, W> MainController<C, W>
where
    C: ConfigRequestService  + Sync + Send + 'static,
    W: WatchService  + Sync + Send + 'static,
{
    pub fn new(config_req_service: Arc<C>, watch_service: Arc<W>) -> Self {
        Self { config_req_service, watch_service }
    }   
    
    
    #[doc = "메인 테스크"]
    pub async fn task_main(&self) {

        let role = self.config_req_service.get_role();

        if role == "master" {
            
            let master_handler = MasterHandler::new(self.config_req_service.clone(), self.watch_service.clone());
            
            match master_handler.run().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        
        } else {
            
            let slave_handler = SlaveHandler::new(self.config_req_service.clone(), self.watch_service.clone());
            
            match slave_handler.run().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        
        }

    }
    
}