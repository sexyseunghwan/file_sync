use crate::common::*;

#[doc = "env 헬퍼함수 정의"]
fn get_env_or_panic(key: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => {
            let msg = format!("[ENV file read Error] '{}' must be set", key);
            log::error!("{}", msg);
            panic!("{}", msg);
        }
    }
}

#[doc = "Function to globally initialize the 'CONFIG_FILE_PATH' variable"]
pub static CONFIG_FILE_PATH: once_lazy<String> = once_lazy::new(|| get_env_or_panic("CONFIG_FILE_PATH"));