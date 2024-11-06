use crate::common::*;

use crate::model::ServerConfig::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Configs {
    server: ServerConfig
}