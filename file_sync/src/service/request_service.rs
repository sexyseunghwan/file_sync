use crate::common::*;


#[async_trait]
pub trait RequestService {

}


#[derive(Debug, Deserialize, Serialize, new)]
pub struct RequestServicePub {
    //pub config: Configs
}


impl RequestService for RequestServicePub {
    
}