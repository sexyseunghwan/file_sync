use crate::common::*;

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
pub fn create_dir_and_file<P: AsRef<Path>, Q: AsRef<Path>>(
    dir_path: P,
    file_name: Q,
) -> Result<PathBuf, anyhow::Error> {
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

#[doc = "디렉토리를 제거해주는 함수"]
/// # Arguments
/// * `path` - 삭제해줄 디렉토리 경로
///
/// # Returns
/// * Result<(), anyhow::Error>
pub fn delete_directory(path: &PathBuf) -> Result<(), anyhow::Error> {
    fs::remove_dir_all(path).map_err(|e| anyhow!("Failed to delete '{:?}' : {:?}", path, e))?;

    info!("Success to delete '{:?}'", path);

    Ok(())
}
