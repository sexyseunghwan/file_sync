use crate::common::*;

#[async_trait]
pub trait RequestService {
    async fn send_info_to_slave(
        &self,
        file_path: &str,
        file_name: &str,
    ) -> Result<(), anyhow::Error>;
    async fn send_info_to_slave_io(
        &self,
        file_path: &str,
        file_name: &str,
        slave_url: Vec<String>,
        secure_mode: bool,
    ) -> Result<(), anyhow::Error>;
    async fn send_info_to_slave_memory(
        &self,
        file_path: &str,
        file_name: &str,
        slave_url: Vec<String>,
        secure_mode: bool,
    ) -> Result<(), anyhow::Error>;
    fn handle_async_function(
        &self,
        task_res: Vec<Result<Result<(), anyhow::Error>, task::JoinError>>,
    ) -> Result<(), anyhow::Error>;
}
