use crate::common::*;

use crate::model::Configs::*;

use crate::utils_modules::io_utils::*;

#[async_trait]
pub trait WatchService {
    fn get_role(&self) -> String;
    //fn get_watcher_delegator() -> Result<>;
}


#[derive(Debug, Deserialize, Serialize)]
pub struct WatchServicePub {
    pub config: Configs
}


impl WatchServicePub {
    
    pub fn new() -> Self {

        let config: Configs = match read_toml_from_file::<Configs>("./Config.toml") {
            Ok(config) => config,
            Err(e) => {
                error!("[Error][WatchServicePub->new()]{:?}", e);
                panic!("{:?}", e)
            }
        };
           
        Self { config }
    }    
}

impl WatchService for WatchServicePub {
    

    #[doc = "해당 프로그램의 역할을 조회해주는 함수"]
    fn get_role(&self) -> String {
        self.config.server.role.clone()
    }

    // #[doc = "docs"]
    // fn get_watcher_delegator() -> Result<> {
        


    // }

}