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
    pub fn task_main() {

        


    }
    

}