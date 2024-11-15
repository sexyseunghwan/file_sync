use crate::common::*;

use crate::utils_modules::time_utils::*;


#[doc = "Json 파일을 읽어서 객체로 변환해주는 함수."]
pub fn read_json_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, anyhow::Error> {
    
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let data = from_reader(reader)?;
    
    Ok(data)
}



#[doc = "toml 파일을 읽어서 객체로 변환해주는 함수"]
pub fn read_toml_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, anyhow::Error> {
    
    let toml_content = fs::read_to_string(file_path)?;
    let toml: T = toml::from_str(&toml_content)?;

    Ok(toml)
}


#[doc = "특정 디렉토리/파일이 존재하지 않는경우 생성해주는 함수."]
pub fn create_dir_and_file(dir_path: &str, file_name: &str) -> Result<PathBuf, anyhow::Error> {

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

#[doc = "특정 파일을 특정 경로에 복사하는 코드"]
pub fn copy_file_for_backup(file_path: PathBuf, dest_folder: &str) -> Result<(), anyhow::Error> {

    let file_name = match file_path.file_name() {
        Some(name) => name.to_str().unwrap_or("default"),
        None => "default",
    };

    let cur_time = get_currnet_utc_naivedatetime();
    let timestamp = get_str_from_naivedatetime(cur_time, "%Y%m%d%H%M%S")?;
    let new_file_name = format!("{}.{}", file_name, timestamp);
    
    /* 새로운 파일 경로 생성 */ 
    let mut dest_path = PathBuf::from(dest_folder);
    dest_path.push(new_file_name);
    
    /* 파일을 복사한다. */
    fs::copy(&file_path, dest_path.as_path())?;

    info!("Backup of file '{}' completed.", &file_name);
    
    Ok(())
}