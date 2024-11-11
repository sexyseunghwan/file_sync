use anyhow::anyhow;

use crate::common::*;

use crate::model::Configs::*;

use crate::utils_modules::io_utils::*;

#[async_trait]
pub trait ConfigService {
    fn get_role(&self) -> String;
    fn get_slave_address(&self) -> Result<Vec<String>, anyhow::Error>;
    fn get_watch_file_list(&self) -> Vec<String>;
    fn get_watch_dir_info(&self) -> String;
    

    //async fn handle_file_change(&self, file_path: &str, tx: &Sender<Result<(), String>>) -> Result<(), anyhow::Error>;
    //fn watch_file(&self) -> Result<(Result<(), SendError<()>>, Receiver<()>), anyhow::Error>;
    //fn get_watcher_delegator() -> Result<>;
}


#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigServicePub {
    pub config: Configs
}


impl ConfigServicePub {
    
    pub fn new() -> Self {

        let config: Configs = match read_toml_from_file::<Configs>("./Config.toml") {
            Ok(config) => config,
            Err(e) => {
                error!("[Error][WatchServicePub->new()]{:?}", e);
                panic!("{:?}", e)
            }
        };
        
        Self { config }
    }    
}

#[async_trait]
impl ConfigService for ConfigServicePub {
    

    #[doc = "감시대상 디렉토리 경로를 반환해주는 함수"]
    fn get_watch_dir_info(&self) -> String {
        self.config.server.watch_path.clone()
    }


    #[doc = "해당 프로그램의 역할을 조회해주는 함수"]
    fn get_role(&self) -> String {
        self.config.server.role.clone()
    }
    
    
    #[doc = "slave_address 정보를 반환하는 함수"]
    fn get_slave_address(&self) -> Result<Vec<String>, anyhow::Error> {

        let slave_address_vec = match self.config.server.slave_address.clone() {
            Some(slave_address_vec) => slave_address_vec,
            None => {
                return Err(anyhow!("[Error][get_slave_address()] "));
            }
        };

        Ok(slave_address_vec)
    }


    #[doc = "감시하는 파일 리스트를 반환해주는 함수"]
    fn get_watch_file_list(&self) -> Vec<String> {

        let watch_path: String = self.config.server.watch_path.clone();

        let watch_file_lists = self.config.server.specific_files
            .iter()
            .map(|file| format!("{}{}", watch_path, file))
            .collect::<Vec<String>>();

        watch_file_lists
        
    }

    // #[doc = ""]
    // async fn handle_file_change(&self, file_path: &str, tx: &Sender<Result<(), String>>) -> Result<(), anyhow::Error> {

        
        

    //     Ok(())
    // }

    // fn watch_file(&self) -> Result<(Result<(), SendError<()>>, Receiver<()>), anyhow::Error> {

    //     /* file 리스트를 가져온다. */
    //     let files = self.config.server.specific_files.clone();
        
    //     /* 지정된 file 리스트가 하나도 없다면 */
    //     if files.len() == 0 {
    //         return Err(anyhow!("[Error][watch_file()] There is no list of files to monitor."))
    //     }
        
        
    //     let mut hotwatch = Hotwatch::new()?;

    //     let (tx, rx) = channel();

    //     let value: Result<(), SendError<()>> = tx.send(());

    //     for file in files.iter() {
         
    //         let file_path = file.to_string();
            
    //         hotwatch.watch(file_path, move |event: Event| {
            
    //             if let WatchEventKind::Modify(_) = event.kind {
    //                 value.expect("Failed to send event");
    //                 //println!("{:?} changed!", event.paths[0]);
    //             }
        
    //         })?
    //     }

    //     Ok((value, rx))
    // }

    // #[doc = "docs"]
    // fn get_watcher_delegator() -> Result<> {
        


    // }

}