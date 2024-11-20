use crate::common::*;


#[doc = ""]
static REQ_CLIENT: once_lazy<Arc<ReqRepositoryPub>> = once_lazy::new(|| {
    initialize_request_clients()
});



#[doc = ""]
pub fn initialize_request_clients() -> Arc<ReqRepositoryPub> {
    
    let client = Client::new();
    Arc::new(ReqRepositoryPub::new(client))

}

#[doc = ""]
pub fn get_request_client() -> Arc<ReqRepositoryPub> {
    let req_client = &REQ_CLIENT;
    Arc::clone(&req_client)
}


#[async_trait]
pub trait ReqRepository {
    
    async fn test(&self) -> Result<(),anyhow::Error>;

}



#[derive(Debug, Getters, Clone, new)]
pub struct ReqRepositoryPub {
    pub client: Client
}



#[async_trait]
impl ReqRepository for ReqRepositoryPub {

    async fn test(&self) -> Result<(), anyhow::Error> {

        println!("{:?}", self.client);
        
        Ok(())
    }    

}