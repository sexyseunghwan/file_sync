use crate::common::*;

use crate::model::Configs::*;

use crate::utils_modules::io_utils::*;

#[async_trait]
pub trait WatchService {
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
    
    // #[doc = "docs"]
    // fn get_watcher_delegator() -> Result<> {
        


    // }

}