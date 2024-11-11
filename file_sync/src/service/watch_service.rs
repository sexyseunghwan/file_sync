use crate::common::*;

use crate::utils_modules::hash_utils::*;

use crate::repository::hash_repository::*;


#[async_trait]
pub trait WatchService {
    
   async fn watch_alarm(&self, file_path_slice: &str) -> (bool, String);     

}



pub struct WatchServicePub {

}



#[async_trait]
impl WatchService for WatchServicePub {
    
    #[doc = "docs"]
    async fn watch_alarm(&self, file_path_slice: &str) -> (bool, String) {
        
        /*  
            현재 이벤트가 걸린 파일의 Hash value 
            - 문제가 발생할 경우 empty vector 반환    
        */  
        let event_hash_val: Vec<u8> = conpute_hash(Path::new(file_path_slice))
            .unwrap_or_else(|_| vec![]);
        
        let storage_hash_guard = get_hash_storage();

        let mut storage_hash = match storage_hash_guard.lock() {
            Ok(storage_hash) => {
                storage_hash
            },
            Err(e) => {
                let err_msg = e.to_string();
                return (false, err_msg)
            }
        };
        
        let storage_hash_val = storage_hash.get_hash(file_path_slice);
        
        if storage_hash_val != event_hash_val {
            storage_hash.update_hash(file_path_slice.to_string(), event_hash_val);
            storage_hash.save();
        }
        
        (true, String::from("test"))
    }

}