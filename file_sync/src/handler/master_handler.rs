use crate::common::*;

use crate::service::file_service::*;
use crate::service::request_service::*;

use crate::configs::Configs::*;

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
                Ok(file_name) => {
                    /* 이벤트가 발생한 파일의 내용이 이전과 다른지 판단하기 위함. */
                    let watch_res = match self.file_service.comparison_file(&file_name) {
                        Ok(watch_res) => watch_res,
                        Err(e) => {
                            error!("{:?}", e);
                            continue;
                        }
                    };

                    /*
                        변경 파일이 있는 경우 -> slave 파일에 변경 파일을 보내준다.
                        - 모니터링 대상인 파일인지 확인해준다.
                        - 모니터링 대상 파일이 아니라면 slave 로 신호를 보내지 않는다.
                    */
                    if watch_res {
                        let monitor_file_list: Vec<String>;
                        {
                            monitor_file_list = get_monitoring_file_detail_path()?;
                        }

                        let mut monitor_yn = false;

                        for inner_file in monitor_file_list {
                            /* 현재 이벤트가 발생한 파일이 내가 모니터링 대상으로 지정한 파일인지 체크해줌 */
                            if file_name == inner_file {
                                monitor_yn = true;
                                break;
                            }
                        }
                        
                        /* 이벤트 발생한 파일의 수정 발생이 확인된 경우 -> Slave server 에 정보를 보내준다.*/
                        if monitor_yn {
                            match self.req_service.send_info_to_slave(&file_name).await {
                                Ok(_) => {
                                    info!(
                                        "Successfully sent files to slave servers. : {}",
                                        &file_name
                                    );
                                    ()
                                }
                                Err(e) => {
                                    error!("{:?}", e);
                                    continue;
                                }
                            }
                        }
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
