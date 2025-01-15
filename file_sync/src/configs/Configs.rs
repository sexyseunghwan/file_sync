use crate::common::*;

use crate::configs::ServerConfig::*;

use crate::utils_modules::io_utils::*;

#[doc = "프로그램 기본 Config 정보를 싱글톤으로 관리하기 위한 전역 변수"]
static CONFIG_INFOS: once_lazy<Arc<RwLock<Configs>>> =
    once_lazy::new(|| initialize_server_configs());

#[doc = "Config 정보를 초기화해주는 함수"]
pub fn initialize_server_configs() -> Arc<RwLock<Configs>> {
    let config: Configs = match read_toml_from_file::<Configs>("./config/Config.toml") {
        Ok(config) => config,
        Err(e) => {
            error!(
                "[Error][WatchServicePub->new()] The config file could not be found. : {:?}",
                e
            );
            panic!("{:?}", e)
        }
    };

    let static_config = Arc::new(RwLock::new(config));
    static_config
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Configs {
    pub server: ServerConfig,
}

#[doc = "config 정보를 반환해주는 함수 - 읽기모드"]
pub fn get_config_read() -> Result<RwLockReadGuard<'static, Configs>, anyhow::Error> {
    CONFIG_INFOS
        .read()
        .map_err(|e| anyhow!("Failed to acquire the read lock due to poisoning: {:?}", e))
}

#[doc = "모니터링 파일의 정확한 위치 리스트를 반환하는 함수"]
pub fn get_monitoring_file_detail_path() -> Result<Vec<String>, anyhow::Error> {
    let config = get_config_read()?;
    let watch_path = config.server.watch_path();
    let file_list = config.server.specific_files();

    let monitor_file_list: Vec<String> = file_list
        .iter()
        .map(|file_path| format!("{}{}", watch_path, file_path))
        .collect();

    Ok(monitor_file_list)
}
