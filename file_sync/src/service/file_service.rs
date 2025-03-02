use crate::common::*;

use crate::utils_modules::hash_utils::*;
use crate::utils_modules::io_utils::*;
use crate::utils_modules::time_utils::*;

use crate::repository::hash_repository::*;

use crate::configs::Configs::*;

#[async_trait]
pub trait FileService {
    fn comparison_file(&self, file_path_slice: &Path) -> Result<bool, anyhow::Error>;
    fn copy_file_for_backup(
        &self,
        backup_target_file_path: PathBuf,
        backup_dir_path: &str,
        modified_file_name: &str
    ) -> Result<(), anyhow::Error>;
    fn backup_file_delete(&self, backup_file_dir: &PathBuf) -> Result<(), anyhow::Error>;
    fn file_event_process(
        &self,
        event: &Event,
        sender: &Sender<Result<String, String>>,
        event_type: &str,
    );
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
    fn comparison_file(&self, target_file_path: &Path) -> Result<bool, anyhow::Error> {
        let target_file_path_str: &str = target_file_path
            .to_str()
            .ok_or_else(|| anyhow!("[Error][comparison_file()] There was a problem converting 'target_file_path' to string."))?;

        /*
            현재 이벤트가 걸린 파일의 Hash value 계산
            - 문제가 발생할 경우 empty vector 반환
        */
        let event_hash_val: Vec<u8> = conpute_hash(target_file_path).unwrap_or_else(|_| vec![]);

        let storage_hash_guard: Arc<Mutex<HashStorage>> = get_hash_storage();
        let mut storage_hash: MutexGuard<'_, HashStorage> = match storage_hash_guard.lock() {
            Ok(storage_hash) => storage_hash,
            Err(e) => return Err(anyhow!("[Error][monitor_file()] {:?}", e)),
        };

        /* 이벤트가 발생한 파일의 기존 저장되어있었던 해쉬값을 가져와준다. => 비교를 위함 */
        let storage_hash_val: Vec<u8> = storage_hash.get_hash(target_file_path_str);

        /* 저장된 해쉬값과 이벤트로 변경된 파일의 해쉬값이 다른경우 */
        if storage_hash_val != event_hash_val {
            storage_hash.update_hash(target_file_path_str.to_string(), event_hash_val);
            storage_hash.save()?;

            info!("The '{}' file has been modified.", target_file_path_str);
            Ok(true) /* 변경 표시 */
        } else {
            info!("The '{}' file has not been modified.", target_file_path_str);
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
        let backup_days: i64;
        {
            let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;
            backup_days = server_config.server.backup_days().clone().unwrap_or(7);
            /* 백업유지기간이 설정이 안되어있다면 기본적으로 7일보존 */
        }

        let entries: fs::ReadDir = fs::read_dir(backup_file_dir)?;

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
                    /* 해당 폴더의 생성일자가 오늘 기준으로 며칠이 되었는지 확인해준다. */
                    let days_diff = match calculate_date_difference_utc(dir_name) {
                        Ok(days_diff) => days_diff,
                        Err(e) => {
                            error!("[Error][backup_file_delete()] {:?}", e);
                            continue;
                        }
                    };

                    /* 날짜가 보관 기간을 넘었을 경우 해당 디렉토리 삭제해준다. */
                    if days_diff >= backup_days {
                        let _delete_res = match delete_directory(&path) {
                            Ok(_) => (),
                            Err(e) => {
                                error!("[Error][backup_file_delete()] {:?}", e);
                                continue;
                            }
                        };
                    }
                }
            }
            /* `path` 가 디렉토리가 아닌 경우의 처리 -> 로깅만 수행 */
            else {
                warn!("It cannot be processed if it is not a `Directory`.");
            }
        }

        Ok(())
    }

    #[doc = "특정 파일을 백업 디렉토리 경로에 복사하는 코드"]
    /// # Arguments
    /// * `backup_target_file_path`     - 동기화 대상이 될 파일 경로
    /// * `backup_dir_path`             - 백업 디렉토리 경로
    /// * `modified_file_name`          - 변경된 파일의 이름
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    fn copy_file_for_backup(
        &self,
        backup_target_file_path: PathBuf,
        backup_dir_path: &str,
        modified_file_name: &str
    ) -> Result<(), anyhow::Error> {

        let file_name: &str = backup_target_file_path
            .file_name()
            .ok_or_else(|| anyhow!("Invalid file name from path"))?
            .to_str()
            .ok_or_else(|| anyhow!("Non-UTF8 file name"))?;
        
        /* 백업 폴더관련 로직 */
        let cur_date: String = get_current_utc_naivedate_str("%Y%m%d")?;
        let timestamp: String = get_current_utc_naivedatetime_str("%Y_%m_%d_%H%M%S")?;
        let new_file_name: String = format!("{}.{}", modified_file_name, timestamp);

        /* 백업 디렉토리 관련 */
        let mut backup_file_path: PathBuf = PathBuf::from(backup_dir_path);
        
        //self.backup_file_delete(&backup_file_path)?; /* 백업 파일 삭제 로직 -> 주기적으로 삭제해주는 함수 */
        
        backup_file_path.push(cur_date);
        backup_file_path = create_dir_and_file(backup_file_path, &new_file_name)?;

        /* 동기화 대상 파일을 백업 디렉토리에 복사한다. */
        fs::copy(&backup_target_file_path, backup_file_path.as_path())?;
        
        info!("Backup of file '{}' completed.", &file_name);
        Ok(())
    }


    #[doc = "파일 이벤트를 처리해주는 함수"]
    /// # Arguments
    /// * `event`   - 모니터링 파일 관련 이벤트  
    /// * `sender`  -  스레드 간 메시지 전달자
    fn file_event_process(
        &self,
        event: &Event,
        sender: &Sender<Result<String, String>>,
        event_type: &str,
    ) {
        match event.paths[0].to_str() {
            Some(file_path) => {
                /*
                    변경이 감지된 파일 경로를 파싱해주는 부분
                    - windows os 의 경우 경로 앞에 의미없는 문자열이 붙는걸 확인. 해당 문자열을 제거해야함.
                */
                let cleaned_path: String = file_path.replace(r"\\?\", "");

                info!(
                    "[event type]: {}, [file name]: {}",
                    event_type, cleaned_path
                );

                sender.send(Ok(cleaned_path.clone())).unwrap_or_else(|err| {
                    error!(
                        "[Error][master_task()] Failed to send error message: {}",
                        err
                    )
                });
            }
            None => {
                sender
                    .send(Err(
                        "[Error][master_task()] Failed to detect file path".to_string()
                    ))
                    .unwrap_or_else(|err| {
                        error!(
                            "[Error][master_task()] Failed to send error message: {}",
                            err
                        )
                    });
            }
        }
    }
}
