use crate::common::*;


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