mod common;
use crate::common::*;

mod utils_modules;
use handler::main_handler;
use handler::main_handler::MainHandler;
use service::watch_service::WatchServicePub;
use utils_modules::logger_utils::*;
use utils_modules::io_utils::*;

mod handler;

mod model;
use model::Configs::*;

mod service;


fn compute_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn hash_file(path: &Path) ->Result<Vec<u8>, anyhow::Error> {
    
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = Vec::new();
    
    // 파일의 메타데이터를 통해 크기를 확인
    let metadata = file.metadata()?;
    if metadata.len() == 0 {
        file = File::open(path)?;   
    }

    file.read_to_end(&mut buffer)?;
    
    //println!("buffer= {:?}", buffer);
    
    hasher.update(&buffer);

    //println!("hasher= {:?}", hasher.clone().finalize().to_vec());

    Ok(hasher.finalize().to_vec())

}


fn calculate_file_hash(file_path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024]; // 파일을 조각으로 읽기

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize().to_vec())
}



#[tokio::main]
async fn main() {

    /* 로깅 시작 */
    set_global_logger();
    info!("File Sync Program Start");
    
    // let config_contents = fs::read_to_string("./Config.toml").unwrap();
    // let configs: Configs = toml::from_str(&config_contents).unwrap();
    
    //let config = read_toml_from_file::<Configs>("./Config.toml").unwrap();
    //println!("{:?}", config);


    /* 종속 서비스 호출 */
    //let watch_service = WatchServicePub::new();
    
    /* 메인핸들러 호출 */
    //let main_handler = MainHandler::new(watch_service);
    
    
    /* 파일 비교 서비스 */


    // let slave_file = File::open("./file_test/slave.txt").unwrap();   /* 원본파일 - slave */
    // let master_file = File::open("./file_test/master.txt").unwrap();  /* 수정파일 - master */
    
    // let slave_lines = BufReader::new(slave_file).lines();
    // let master_lines = BufReader::new(master_file).lines();
    
    // let mut changes = Vec::new(); /* 변경분 저장 벡터 */
    
    // let mut slave_lines_iter = slave_lines.enumerate().peekable();
    // let mut master_lines_iter = master_lines.enumerate().peekable();

    // /* 
    //     master file, slave file 내부의 데이터가 존재하는 경우 계속 while 문으로 돌려준다.
    //     내부적으로 cursor 형식으로 비교를 수행한다.
    // */
    // while slave_lines_iter.peek().is_some() || master_lines_iter.peek().is_some() {
        
    //     /* master file, slave file 최상단 데이터 */
    //     match (slave_lines_iter.peek(), master_lines_iter.peek()) {
            
    //         /* master file, slave file 둘다 line 에 데이터가 있는 경우*/
    //         (Some(slave), Some(master)) => {
    //             let (slave_idx, slave_line) = slave;
    //             let (master_idx, master_line) = master;

    //             /*  */
    //             let slave_line_data = slave_line.as_ref().unwrap();
    //             let master_line_data = master_line.as_ref().unwrap();
                
    //             if slave_line_data != master_line_data {
    //                 println!("{} // {}", master_idx, master_line_data);
    //             }

    //             //println!("orig_line= {:?}", orig_line);
    //             //println!("modif_line= {:?}", modif_line);

    //             /* cursor 를 하나 늘려준다. */
    //             slave_lines_iter.next();
    //             master_lines_iter.next();

    //         },
    //         /* slave file line 에만 데이터가 있는 경우 */
    //         (Some(orig), None) => {
    //             let (orig_idx, orig_line) = orig;
    //             let orig_line = orig_line.as_ref().unwrap();
                
    //             // 원본 파일에는 있지만 수정된 파일에는 없는 경우
    //             changes.push((*orig_idx, orig_line.clone(), String::from("[Deleted]")));
    //             //println!("");
    //             // slave file에는 없지만, master file에는 있는경우.
    //             slave_lines_iter.next();
    //         },
            
    //         /* master file line 에만 데이터가 있는 경우 */
    //         (None, Some(modif)) => {
    //             let (modif_idx, modif_line) = modif;
    //             let modif_line = modif_line.as_ref().unwrap();
                
    //             // 수정된 파일에는 있지만 원본 파일에는 없는 경우
    //             //changes.push((*modif_idx, String::from("[Added]"), modif_line.clone()));
    //             println!("");
    //             // master file 에는 있지만, slave file 에는 없는 경우.
    //             master_lines_iter.next();
    //         },
    //         (None, None) => break,
    //     }
    // }
    
    // while original_lines_iter.peek().is_some() || modified_lines_iter.peek().is_some() {
        
    //     match (original_lines_iter.peek(), modified_lines_iter.peek()) {
            
    //         (Some(orig), Some(modif)) => {
    //             let (orig_idx, orig_line) = orig;
    //             let (modif_idx, modif_line) = modif;

    //             let orig_line = orig_line.as_ref().unwrap();
    //             let modif_line = modif_line.as_ref().unwrap();

    //             if compute_hash(orig_line) != compute_hash(modif_line) {
    //                 changes.push((*modif_idx, orig_line.clone(), modif_line.clone()));
    //             }

    //             original_lines_iter.next();
    //             modified_lines_iter.next();
    //         },
    //         (Some(orig), None) => {
    //             let (orig_idx, orig_line) = orig;
    //             let orig_line = orig_line.as_ref().unwrap();
    //             // 원본 파일에는 있지만 수정된 파일에는 없는 경우
    //             changes.push((*orig_idx, orig_line.clone(), String::from("[Deleted]")));
    //             original_lines_iter.next();
    //         },
    //         (None, Some(modif)) => {
    //             let (modif_idx, modif_line) = modif;
    //             let modif_line = modif_line.as_ref().unwrap();
    //             // 수정된 파일에는 있지만 원본 파일에는 없는 경우
    //             changes.push((*modif_idx, String::from("[Added]"), modif_line.clone()));
    //             modified_lines_iter.next();
    //         },
    //         (None, None) => break,
    //     }
    // }

    //notify = "7.0.0"
    // for elem in changes {
    //     println!("{:?}", elem);
    // }
    
    /* ============== HOT Watch ============== */
    let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
    
    let file = "./file_test/master.txt";
    let mut last_hash = hash_file(Path::new(file)).unwrap();
    
    let (tx, rx) = channel();

    hotwatch.watch(file, move |event: Event| {
        
        if let WatchEventKind::Modify(_) = event.kind {
            tx.send(()).expect("Failed to send event");
            println!("{:?} changed!", event.paths[0]);
        }

    }).expect("failed to watch file!");
    
    
    loop {
        
        rx.recv().unwrap();
        
        let cur_hash = hash_file(Path::new(file)).unwrap();

        /* 파일 해시값이 다른경우 수정으로 봄 */
        if cur_hash != last_hash {
            println!("Detected a change!");
            last_hash = cur_hash;
        }
        
        // 필요에 따라 추가 로직 실행
        
    }


    /* watch 관련 서비스 */
    // let (tx, rx) = channel();
    
    // let config = Config::default()
    //     .with_poll_interval(Duration::from_secs(2));

    // // Watcher 인스턴스 생성
    // let mut watcher: RecommendedWatcher = Watcher::new(tx, config).unwrap();
    // let file_path= "./file_test/master.txt";
    // let mut last_hash = hash_file(Path::new(file_path)).unwrap();

    // //println!("last_hash: {:?}", last_hash);

    // // 관찰할 디렉토리 설정
    // watcher.watch(Path::new(file_path), RecursiveMode::Recursive).unwrap();

    // //println!("Watching for changes...");
    
    // loop {
    //     match rx.recv() {
    //         Ok(_event) => {

    //             println!("event");
    //             //println!("{:?}", _event);
    //             // let cur_hash = hash_file(Path::new(file_path)).unwrap();
                
    //             // println!("cur_hash: {:?}", cur_hash);
    //             // println!("last_hash: {:?}", last_hash);
                
    //             // if cur_hash != last_hash {
    //             //     println!("modified file");
    //             //     last_hash = cur_hash;
    //             // }
    //         },
    //         Err(e) => println!("watch error: {:?}", e),
    //     }
    // }
    
    // loop {
    //     match rx.recv() {
    //         Ok(event) => {

    //             let evt = event.unwrap().clone();

    //             if let Some(path) = evt.paths.first() {
    //                 match evt.kind {
    //                     EventKind::Modify(_) => {
    //                         //println!("Modified: {:?}", path)
    //                         let cur_hash = hash_file(Path::new(file_path)).unwrap();
    //                         println!("{:?}", cur_hash);
    //                     },
    //                     _ => (), // Ignore other events
    //                 }
    //             }
                
    //             // if let Some(path) = event.clone().expect("REASON").paths.first() {
    //             //     match event.unwrap().kind {
    //             //         EventKind::Modify(_) => println!("Modified: {:?}", path),
    //             //         _ => (), // Ignore other events
    //             //     }
    //             // }

    //             //println!("{:?}", event)
    //             //event.wrtie
                
    //         },
    //         // Ok(event) => match event {
    //         //     DebouncedEvent::Write(path) => println!("Modified: {:?}", path),
    //         //     _ => (), // 다른 모든 이벤트는 무시
    //         // },
    //         Err(e) => println!("watch error: {:?}", e),
    //     }
    // }
}