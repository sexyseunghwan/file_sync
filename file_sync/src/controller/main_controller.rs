use crate::common::*;

use crate::service::file_service::*;
use crate::service::request_service::*;

use crate::handler::master_handler::*;
use crate::handler::slave_handler::*;

use crate::configs::Configs::*;

#[derive(Debug)]
pub struct MainController<R, F>
where
    R: RequestService + Sync + Send + 'static,
    F: FileService + Sync + Send + 'static,
{
    req_service: Arc<R>,
    file_service: Arc<F>,
}

impl<R, F> MainController<R, F>
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

    #[doc = "메인 테스크"]
    pub async fn task_main(&self) {
        /* System role */
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
            let master_handler =
                MasterHandler::new(self.req_service.clone(), self.file_service.clone());

            match master_handler.run().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        } else {
            /* System role 이 slave 인 경우 */

            let slave_handler =
                SlaveHandler::new(self.req_service.clone(), self.file_service.clone());

            match slave_handler.run().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }
}
