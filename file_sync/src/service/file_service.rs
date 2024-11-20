use crate::common::*;

use crate::utils_modules::hash_utils::*;
use crate::utils_modules::io_utils::*;
use crate::utils_modules::time_utils::*;

use crate::repository::hash_repository::*;

use crate::configs::Configs::*;


#[async_trait]
pub trait FileService {
    fn comparison_file(&self, file_path_slice: &str) -> Result<bool, anyhow::Error>;
    fn copy_file_for_backup(&self, backup_target_file_path: PathBuf, backup_dir_path: &str) -> Result<(), anyhow::Error>;
    fn backup_file_delete(&self, backup_file_dir: &PathBuf) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Deserialize, Serialize, new)]
pub struct FileServicePub {}


#[async_trait]
impl FileService for FileServicePub {
    
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


    
    #[doc = "백업 디렉토리를 주기적으로 제거해주는 함수"]
    /// # Arguments
    /// * `backup_file_dir` - 백업 디렉토리 경로
    /// 
    /// # Returns
    /// * Result<(), anyhow::Error>
    fn backup_file_delete(&self, backup_file_dir: &PathBuf) -> Result<(), anyhow::Error> {
        
        /* 백업 유지기간 */
        let backup_days;
        {
            let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;
            backup_days = server_config
                .server
                .backup_days()
                .clone()
                .unwrap_or(7);
        }
        
        let entries = fs::read_dir(backup_file_dir)?;
        
        for entry in entries {
            
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    error!("[Error][backup_file_delete()] There is a problem with the value of 'entry'. : {:?}", e);
                    continue;
                }
            };
            
            /* 백업 디렉토리 날짜별 폴더 경로 */
            let path: PathBuf = entry.path();
            
            /* `path`가 디렉토리인 경우의 처리 */ 
            if path.is_dir() {
                
                if let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) {

                    /* 해당 폴더의 생성일자가 오늘 기준으로 몇일이 되었는지 확인해준다. */
                    let days_diff = match calculate_date_difference_utc(dir_name){
                        Ok(days_diff) => days_diff,
                        Err(e) => {
                            error!("[Error][backup_file_delete()] {:?}", e);
                            continue
                        }
                    };
                    
                    /* 날짜가 보관 기간을 넘었을 경우 해당 디렉토리 삭제해준다. */
                    if days_diff >= backup_days {
                        let _delete_res = match delete_directory(&path) {
                            Ok(_) => (),
                            Err(e) => {
                                error!("[Error][backup_file_delete()] {:?}", e);
                                continue
                            }
                        };
                    }
                }
            }
        }
        
        Ok(())
    }


    #[doc = "특정 파일을 백업 디렉토리 경로에 복사하는 코드"]
    /// # Arguments
    /// * `backup_target_file_path`     - 동기화 대상이 될 파일 경로 
    /// * `backup_dir_path`             - 백업 디렉토리 경로
    /// 
    /// # Returns
    /// * Result<(), anyhow::Error>
    fn copy_file_for_backup(&self, backup_target_file_path: PathBuf, backup_dir_path: &str) -> Result<(), anyhow::Error> {

        let file_name = backup_target_file_path.file_name()
            .ok_or_else(|| anyhow!("Invalid file name from path"))?
            .to_str()
            .ok_or_else(|| anyhow!("Non-UTF8 file name"))?;
        

        /* 백업 폴더관련 로직 */
        let cur_date = get_current_utc_naivedate_str("%Y%m%d")?;
        let timestamp = get_current_utc_naivedatetime_str("%Y_%m_%d_%H%M%S")?;
        let new_file_name = format!("{}.{}", file_name, timestamp);
        
        /* 백업 디렉토리 관련 */ 
        let mut backup_file_path = PathBuf::from(backup_dir_path);
        
        self.backup_file_delete(&backup_file_path)?; /* 백업 파일 삭제 로직 */
        
        backup_file_path.push(cur_date);
        backup_file_path = create_dir_and_file(backup_file_path, &new_file_name)?;
        
        
        /* 동기화 대상 파일을 백업 디렉토리에 복사한다. */
        fs::copy(&backup_target_file_path, backup_file_path.as_path())?;
        
        info!("Backup of file '{}' completed.", &file_name);
        Ok(())
    }


    


}