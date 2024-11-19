use crate::common::*;

use crate::configs::Configs::*;

use crate::utils_modules::time_utils::*;


#[doc = "Json 파일을 읽어서 객체로 변환해주는 함수."]
/// # Arguments
/// * `file_path` - 읽을대상 파일이 존재하는 경로
/// 
/// # Returns
/// * Result<T, anyhow::Error> - 성공적으로 파일을 읽었을 경우에는 json 호환 객체를 반환해준다.
pub fn read_json_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, anyhow::Error> {
    
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let data = from_reader(reader)?;
    
    Ok(data)
}



#[doc = "toml 파일을 읽어서 객체로 변환해주는 함수"]
/// # Arguments
/// * `file_path` - 읽을 대상 toml 파일이 존재하는 경로
/// 
/// # Returns
/// * Result<T, anyhow::Error> - 성공적으로 파일을 읽었을 경우에는 json 호환 객체를 반환해준다.
pub fn read_toml_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, anyhow::Error> {
    
    let toml_content = fs::read_to_string(file_path)?;
    let toml: T = toml::from_str(&toml_content)?;

    Ok(toml)
}


#[doc = "특정 디렉토리/파일이 존재하지 않는경우 생성해주는 함수."]
/// # Arguments
/// * `dir_path`  - 파일이 위치할 디렉토리의 경로를 나타낸다
/// * `file_name` - 생성할 파일의 이름을 나타낸다
/// 
/// # Returns
/// * `Result<PathBuf, anyhow::Error>` - 성공적으로 파일이 생성되었을 때 해당 파일의 경로를 포함하는 `PathBuf` 객체를 반환.
pub fn create_dir_and_file<P: AsRef<Path>, Q: AsRef<Path>>(dir_path: P, file_name: Q) -> Result<PathBuf, anyhow::Error> {

    let dir_path = dir_path.as_ref();
    let file_name = file_name.as_ref();

    let path = Path::new(dir_path);

    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    let file_path: PathBuf = path.join(file_name);

    if !file_path.exists() {
        let _file = File::create(&file_path)?;
    }
    
    Ok(file_path)
}



#[doc = "특정 파일을 백업 디렉토리 경로에 복사하는 코드"]
/// # Arguments
/// * `backup_target_file_path`     - 동기화 대상이 될 파일 경로 
/// * `backup_dir_path`             - 백업 디렉토리 경로
/// 
/// # Returns
/// * Result<(), anyhow::Error>
pub fn copy_file_for_backup(backup_target_file_path: PathBuf, backup_dir_path: &str) -> Result<(), anyhow::Error> {

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
    
    backup_file_delete(&backup_file_path)?; /* 백업 파일 삭제 로직 */
    
    backup_file_path.push(cur_date);
    backup_file_path = create_dir_and_file(backup_file_path, &new_file_name)?;
    

    /* 동기화 대상 파일을 백업 디렉토리에 복사한다. */
    fs::copy(&backup_target_file_path, backup_file_path.as_path())?;
    
    info!("Backup of file '{}' completed.", &file_name);
    Ok(())
}




#[doc = "디렉토리를 제거해주는 함수"]
/// # Arguments
/// * `path` - 삭제해줄 디렉토리 경로
/// 
/// # Returns
/// * Result<(), anyhow::Error>
fn delete_directory(path: &PathBuf) -> Result<(), anyhow::Error> {
    
    fs::remove_dir_all(path)
        .map_err(|e| anyhow!("Failed to delete '{:?}' : {:?}", path, e))?;
    
    info!("Success to delete '{:?}'", path);
    
    Ok(())
}



#[doc = "백업 디렉토리를 주기적으로 제거해주는 함수"]
/// # Arguments
/// * `backup_file_dir` - 백업 디렉토리 경로
/// 
/// # Returns
/// * Result<(), anyhow::Error>
fn backup_file_delete(backup_file_dir: &PathBuf) -> Result<(), anyhow::Error> {
    
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