use crate::common::*;

use crate::service::request_service::*;
use crate::service::watch_service::*;

use crate::handler::master_handler::*;
use crate::handler::slave_handler::*;

use crate::configs::Configs::*;


#[derive(Debug)]
pub struct MainController<R, W>
where
    R: RequestService + Sync + Send + 'static,
    W: WatchService + Sync + Send + 'static,
{
    req_service: Arc<R>,
    watch_service: Arc<W>,
}


impl<R, W> MainController<R, W>
where
    R: RequestService  + Sync + Send + 'static,
    W: WatchService  + Sync + Send + 'static,
{
    pub fn new(req_service: Arc<R>, watch_service: Arc<W>) -> Self {
        Self { req_service, watch_service }
    }   
    
    
    #[doc = "메인 테스크"]
    pub async fn task_main(&self) {

        let role;
        {
            let server_config = match get_config_read() {
                Ok(server_config) => server_config,
                Err(e) => {
                    error!("[Error][task_main()] {:?}", e);
                    panic!("{:?}", e)
                }
            };

            role = server_config.server.role().to_string();
        }
        
        /* System role 이 master 인 경우 */
        if role == "master" {
            
            let master_handler = MasterHandler::new(
                self.req_service.clone(), 
                self.watch_service.clone()
            );
            
            match master_handler.run().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        
        } else {
        /* System role 이 slave 인 경우 */
        
            let slave_handler = SlaveHandler::new(
                self.req_service.clone(), 
                self.watch_service.clone()
            );
            
            match slave_handler.run().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }
}