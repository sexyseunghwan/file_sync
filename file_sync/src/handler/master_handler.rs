use crate::common::*;

use crate::service::file_service::*;
use crate::service::request_service::*;

use crate::configs::Configs::*;

use crate::model::monitoring_path_info::*;

#[derive(Debug)]
pub struct MasterHandler<R, F>
where
    R: RequestService + Sync + Send + 'static,
    F: FileService + Sync + Send + 'static,
{
    req_service: Arc<R>,
    file_service: Arc<F>,
}

impl<R, F> MasterHandler<R, F>
where
    R: RequestService + Sync + Send + 'static,
    F: FileService + Sync + Send + 'static,
{
    pub fn new(req_service: Arc<R>, file_service: Arc<F>) -> Self {
        Self {
            req_service,
            file_service,
        }
    }

    #[doc = "프로그램 role 이 master 인경우의 작업"]
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        /* 감시파일 경로 */
        let watch_dir_path: String;
        {
            let server_config: RwLockReadGuard<'_, Configs> = get_config_read()?;
            watch_dir_path = server_config.server.get_watch_dir_path();
        }

        let mut hotwatch: Hotwatch = Hotwatch::new()?;

        /* 해당 파일을 계속 감시해준다. */
        let (tx, rx) = channel::<Result<String, String>>();

        let self_file_service: Arc<F> = self.file_service.clone(); /* self.file_service 의 참조 복사본 생성 */

        /* tx 부분 - 파일변경 감시해주는 부분 */
        hotwatch.watch(watch_dir_path, move |event: Event| match &event.kind {
            WatchEventKind::Modify(_) => {
                self_file_service.file_event_process(&event, &tx, "Modify");
            }
            WatchEventKind::Create(_) => {
                self_file_service.file_event_process(&event, &tx, "Create");
            }
            WatchEventKind::Remove(_) => {

                self_file_service.file_event_process(&event, &tx, "Remove");
            }
            _ => {
                warn!(
                    "[Warn][master_handler -> main()] Undetectable event: kind = {:?}, paths = {:?}",
                    &event.kind, &event.paths
                )
            }
        })?;

        /*
            rx 부분 - receive 를 계속 감시한다.
        */
        for received in rx {
            match received {
                Ok(file_path) => {

                    /* 일단 해당 파일이 모니터링 대상인지 확인을 먼저 함 */
                    let monitor_file_list: Vec<MonitoringPathInfo>;
                    {
                        monitor_file_list = get_monitoring_file_detail_path()?;
                    }

                    let mut monitor_yn: bool = false;
                    let mut short_file_path: String = String::new();                 

                    for inner_file in monitor_file_list {
                        let inner_file_path: &Path = Path::new(inner_file.full_file_path());
                        let file_path: &Path = Path::new(&file_path);

                        /* 현재 이벤트가 발생한 파일이 내가 모니터링 대상으로 지정한 파일인지 체크해줌 */
                        if inner_file_path == file_path {
                            monitor_yn = true;
                            short_file_path = inner_file.file_path().to_string();
                            break;
                        }
                    }
                    
                    
                    if monitor_yn {
                        /* 모니터링 대상 파일이 맞는 경우 */
                        let file_name_path: &Path = Path::new(&file_path);

                        /* 이벤트가 발생한 파일의 내용이 이전과 다른지 판단하기 위함. */
                        let modify_yn: bool = match self.file_service.comparison_file(file_name_path) {
                            Ok(watch_res) => watch_res,
                            Err(e) => {
                                error!("[Error][run() -> watch_res]{:?}", e);
                                continue;
                            }
                        };
                        
                        if modify_yn {
                            match self.req_service.send_info_to_slave(&file_path, &short_file_path).await {
                                Ok(_) => {
                                    info!(
                                        "Successfully sent files to slave servers. : {}",
                                        &short_file_path
                                    );
                                }
                                Err(e) => {
                                    error!("[Error][run() -> modify_yn] {:?}", e);
                                    continue;
                                }
                            }

                        } else {
                            info!("This file has not been modified.: {}", &file_path);
                        }

                    } else {
                        /* 모니터링 대상 파일이 아닌 경우 */
                        info!("The file '{}' is not a monitoring target file.", &file_path);
                    }
                }
                Err(e) => {
                    error!(
                        "[Error][master_task()] Error processing file change: {:?}",
                        e
                    );
                    /* 에러 발생시 다음 수신으로 전환 */
                    continue;
                }
            }
        }

        Ok(())
    }
}
