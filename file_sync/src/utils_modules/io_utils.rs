use core::{error, panic};
use std::str::FromStr;

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