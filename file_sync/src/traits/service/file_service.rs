use crate::common::*;

#[async_trait]
pub trait FileService {
    fn comparison_file(&self, file_path_slice: &Path) -> Result<bool, anyhow::Error>;
    fn copy_file_for_backup(
        &self,
        backup_target_file_path: PathBuf,
        backup_dir_path: &str,
        modified_file_name: &str,
    ) -> Result<(), anyhow::Error>;
    fn backup_file_delete(&self, backup_file_dir: &Path) -> Result<(), anyhow::Error>;
    fn file_event_process(
        &self,
        event: &Event,
        sender: &Sender<Result<String, String>>,
        event_type: &str,
    );
}
