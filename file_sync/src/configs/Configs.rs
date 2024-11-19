use crate::common::*;

use crate::configs::ServerConfig::*;

use crate::utils_modules::io_utils::*;

#[doc = "프로그램 기본 Config 정보를 싱글톤으로 관리하기 위한 전역 변수"]
static CONFIG_INFOS: once_lazy<Arc<RwLock<Configs>>> = once_lazy::new(|| {
    initialize_server_configs()
});


#[doc = "Config 정보를 초기화해주는 함수"]
pub fn initialize_server_configs() -> Arc<RwLock<Configs>> {
    
    let config: Configs = match read_toml_from_file::<Configs>("./config/Config.toml") {
        Ok(config) => config,
        Err(e) => {
            error!("[Error][WatchServicePub->new()] The config file could not be found. : {:?}", e);
            panic!("{:?}", e)
        }
    };
    
    let static_config = Arc::new(RwLock::new(config));    
    static_config
}


#[derive(Debug, Deserialize, Serialize)]
pub struct Configs {
    pub server: ServerConfig
}

#[doc = "config 정보를 반환해주는 함수"]
pub fn get_server_configs() -> Arc<RwLock<Configs>>
{
    let config_info = &CONFIG_INFOS;
    Arc::clone(&config_info)
}


#[doc = "config 정보를 반환해주는 함수 - 읽기모드"]
pub fn get_config_read() -> Result<RwLockReadGuard<'static, Configs>, anyhow::Error> {

    CONFIG_INFOS
        .read()
        .map_err(|e| anyhow!("Failed to acquire the read lock due to poisoning: {:?}", e))

}


#[doc = "config 정보를 반환해주는 함수 - 쓰기모드"]
pub fn get_config_write() -> Result<RwLockWriteGuard<'static, Configs>, anyhow::Error> {

    CONFIG_INFOS
        .write()
        .map_err(|e| anyhow!("Failed to acquire the read lock due to poisoning: {:?}", e))

}