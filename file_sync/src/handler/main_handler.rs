use crate::common::*;

use crate::service::watch_service::*;


#[derive(Debug, Deserialize, Serialize)]
pub struct MainHandler<W: WatchService> {
    watch_service: W
}


impl<W: WatchService> MainHandler<W> {

    pub fn new(watch_service: W) -> Self {
        Self { watch_service }
    }

    #[doc = "docs"]
    pub async fn task_main(&self) {

        let role = self.watch_service.get_role();
        
        if role == "master" {
            self.master_task().await;
        } else {
            self.slave_task().await;
        }
        
    }
    
    
    #[doc = "docs"]
    pub async fn master_task(&self) {

    } 
    

    #[doc = "docs"]
    pub async fn slave_task(&self) {
        
    }

}