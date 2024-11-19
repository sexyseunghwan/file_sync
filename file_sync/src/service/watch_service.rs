use crate::common::*;

use crate::utils_modules::hash_utils::*;

use crate::repository::hash_repository::*;


#[async_trait]
pub trait WatchService {
    fn comparison_file(&self, file_path_slice: &str) -> Result<bool, anyhow::Error>;
}

#[derive(Debug, Deserialize, Serialize, new)]
pub struct WatchServicePub {}


#[async_trait]
impl WatchService for WatchServicePub {

    #[doc = "변경된 파일을 기존 파일과 비교하는 함수"]
    /// # Arguments
    /// * `target_file_path` - 변화가 생긴 파일의 경로
    /// 
    /// # Returns
    /// * Result<bool, anyhow::Error> - 파일의 변화가 있는 경우에 True, 변화가 없는 경우에는 False
    fn comparison_file(&self, target_file_path: &str) -> Result<bool, anyhow::Error> {
        
        /*  
            현재 이벤트가 걸린 파일의 Hash value 
            - 문제가 발생할 경우 empty vector 반환    
        */  
        let event_hash_val: Vec<u8> = conpute_hash(Path::new(target_file_path))
            .unwrap_or_else(|_| vec![]);
        
        let storage_hash_guard = get_hash_storage();
        let mut storage_hash = match storage_hash_guard.lock() {
            Ok(storage_hash) => {
                storage_hash
            },
            Err(e) => {
                return Err(anyhow!("[Error][monitor_file()] {:?}", e))
            }
        };
        
        let storage_hash_val = storage_hash.get_hash(target_file_path);
        
        /* 저장된 해쉬값과 이벤트로 변경된 파일의 해쉬값이 다른경우 */
        if storage_hash_val != event_hash_val {
            
            storage_hash.update_hash(target_file_path.to_string(), event_hash_val);
            storage_hash.save()?;
            
            info!("The '{}' file has been modified.", target_file_path);
            Ok(true) /* 변경 표시 */
        } else {
            info!("The '{}' file has not been modified.", target_file_path);
            Ok(false) /* 변경없음 표시 */
        }

    }
}