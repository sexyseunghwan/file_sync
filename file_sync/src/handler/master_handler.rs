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
        /* 감시할 파일 리스트 */
        // let watch_dir_path, watch_file_vec;
        // {
        //     let server_config = get_config_read()?;
        //     watch_file_vec = server_config.server.get_watch_file_list();
        //     watch_dir_path = server_config.server.get_watch_dir_path();

        //     (watch_dir_path, watch_file_vec)
        // }

        /* 감시파일 경로 */
        let watch_dir_path;
        {
            let server_config = get_config_read()?;
            watch_dir_path = server_config.server.get_watch_dir_path();
        }

        let mut hotwatch = Hotwatch::new()?;

        /* 해당 파일을 계속 감시해준다. */
        let (tx, rx) = channel::<Result<String, String>>();

        let self_file_service = self.file_service.clone(); /* self.file_service 의 참조 복사본 생성 */

        /* tx 부분 */
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
        
        // for monitor_file in &slave_address_vec {
        //     //let file_path = monitor_file.to_string();
        //     //let file_dir =
        //     let tx_clone: Sender<Result<String, String>> = tx.clone(); /* tx의 복사본을 생성 -> 소유권 때문 위의 for 문 때문임. */
        //     let self_file_service = self.file_service.clone(); /* self.file_service 의 참조 복사본 생성 */
        //     hotwatch.watch(file_path.clone(), move |event: Event| {

        //         match &event.kind {
        //             WatchEventKind::Modify(_) => {
        //                 self_file_service.file_event_process(&event, &tx_clone);
        //             }
        //             WatchEventKind::Create(_) => {
        //                 self_file_service.file_event_process(&event, &tx_clone);
        //             }
        //             WatchEventKind::Remove(_) => {
        //                 self_file_service.file_event_process(&event, &tx_clone);
        //             }
        //             _=> {
        //                 error!("[Error][master_handler -> main()] This is an undetectable event.")
        //             }
        //         }

        //         // if let WatchEventKind::Modify(_) = event.kind {
        //         //     match event.paths[0].to_str() {
        //         //         Some(file_path) => {
        //         //             info!("change path: {:?}", file_path);

        //         //             /*
        //         //                 변경이 감지된 파일 경로를 파싱해주는 부분
        //         //                 - windows os 의 경우 경로 앞에 의미없는 문자열이 붙는걸 확인. 해당 문자열을 제거해야함.
        //         //             */
        //         //             let cleaned_path = file_path.replace(r"\\?\", "");

        //         //             tx_clone
        //         //                 .send(Ok(cleaned_path.clone()))
        //         //                 .unwrap_or_else(|err| {
        //         //                     error!(
        //         //                         "[Error][master_task()] Failed to send error message: {}",
        //         //                         err
        //         //                     )
        //         //                 });
        //         //         }
        //         //         None => {
        //         //             tx_clone
        //         //                 .send(Err(
        //         //                     "[Error][master_task()] Failed to detect file path".to_string()
        //         //                 ))
        //         //                 .unwrap_or_else(|err| {
        //         //                     error!(
        //         //                         "[Error][master_task()] Failed to send error message: {}",
        //         //                         err
        //         //                     )
        //         //                 });
        //         //         }
        //         //     }
        //         // }
        //     })?;
        //}

        /*
            rx 부분 - for문을 통해서 receive 를 계속 감시한다.
        */
        for received in rx {
            match received {
                Ok(file_name) => {
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

                        for file in monitor_file_list {
                            if file_name == file {
                                monitor_yn = true;
                                break;
                            }
                        }

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
