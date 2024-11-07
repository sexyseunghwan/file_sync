use crate::common::*;

use crate::service::config_service::*;


#[derive(Debug, Deserialize, Serialize)]
pub struct MainHandler<C: ConfigService> {
    config_service: C
}


impl<C: ConfigService> MainHandler<C> {

    pub fn new(config_service: C) -> Self {
        Self { config_service }
    }

    #[doc = "해당 프로그램이 master role 인지 slave role 인지 정해준다."]
    pub async fn task_main(&self) {

        let role = self.config_service.get_role();
        
        if role == "master" {
            
            match self.master_task().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                }
            }

        } else {
            
            match self.slave_task().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }
    
    
    #[doc = "프로그램 role 이 master 인경우의 작업"]
    pub async fn master_task(&self) -> Result<(), anyhow::Error> {   
        
        let mut hotwatch = Hotwatch::new()?;

        /* 감시할 파일 리스트 */
        let slave_address_vec = self.config_service.get_watch_file_list();
        
        /* 해당 파일을 계속 감시해준다. */
        let (_tx, rx) = channel();
        
        for file in slave_address_vec.iter() {
         
            let file_path = file.to_string();
    
            hotwatch.watch(file_path, move |event: Event| {
                
                if let WatchEventKind::Modify(_) = event.kind {
                    //value
                    println!("{:?} changed!", event.paths[0]);
                }
                
            })?
        }
        
        /* 변경이 일어난 경우 slave 시스템으로 정보를 보냄. */
        loop {
            rx.recv()?
        }
        
    }   
    

    #[doc = "프로그램 role 이 slave 인경우의 작업"]
    pub async fn slave_task(&self) -> Result<(), anyhow::Error> {

        

        Ok(())
    }

}