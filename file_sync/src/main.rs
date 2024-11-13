mod common;
use core::panic;

use crate::common::*;

mod utils_modules;
use handler::main_handler;
use handler::main_handler::*;

use service::config_request_service::*;

use service::watch_service::*;

use utils_modules::logger_utils::*;
use utils_modules::io_utils::*;

mod handler;

mod model;
use model::Configs::*;

mod service;

mod repository;


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


async fn test() -> Result<(), anyhow::Error> {
    
    let url = "http://192.168.8.77:9000/upload";
    let file_path = "./file_test/master.txt";
    
    let file = TokioFile::open(file_path).await?;
    let reader = FramedRead::new(file, BytesCodec::new());
    let stream = 
        reader
            .map_ok(|bytes| bytes.freeze())
            .map_err(|e| std::io::Error::new( std::io::ErrorKind::Other, e));
    
    let body = Body::wrap_stream(stream);

    let client = Client::new();

    let response = client.post(url)
        .body(body)
        .send()
        .await?;

    if response.status().is_success() {
        println!("File was sent successfully.");
    } else {
        println!("Failed to send file: {}", response.status());
    }

    
    //let body = Body::wrap_stream(stream); 

    //let mut buffer = Vec::new();
    /* 파일의 모든 내용을 읽어서 버퍼에 저장 */ 
    //file.read_to_end(&mut buffer).await.expect("Failed to read file");

    /* Bytes 스트림을 생성 */ 
    // let stream = FramedRead::new(file, BytesCodec::new())
    //     .map_ok(|bytes| bytes.freeze())
    //     .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));


    //let stream = Cursor::new(buffer);
    

    //let client = Client::new();
    //let body = Body::(stream);

    // let mut file = TokioFile::open(file_path).await.expect("Failed to open file");
    // let mut buffer = Vec::new();
    



    // /* 파일의 모든 내용을 읽어서 버퍼에 저장 */ 
    // file.read_to_end(&mut buffer).await.expect("Failed to read file");

    // //let stream = Cursor::new(buffer);
    
    
    // let body = Body::wrap(stream);
    
    // let client = Client::new(); /* Clinet 를 전역적으로 한번만 셋팅하면 될거 같은데. */
    // let part = multipart::Part::stream(body)
    //     .file_name(file_path.to_string().to_owned());

    // let form = multipart::Form::new().part("file", part);

    // let response = client.post(url)
    //     .multipart(form)
    //     .send()
    //     .await?;
    
    // if response.status().is_success() {
    //     println!("File was sent successfully.");
    // } else {
    //     eprintln!("Failed to send file: {:?}", response.status());
    // }
    
    Ok(())
}

async fn actix_main() -> Result<(), anyhow::Error> {

    HttpServer::new(|| {
        App::new()
            .service(upload)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;



    Ok(())

}

#[post("/upload")]
async fn upload(mut payload: web::Payload) -> impl Responder {
    
    let mut file = File::create("uploaded_file.txt").expect("Failed to create file");

    /* 스트림에서 데이터를 읽고 파일에 쓴다. */ 
    while let Ok(Some(chunk)) = payload.try_next().await {
        let data = chunk;
        let _ = file.write_all(&data);
    }

    HttpResponse::Ok().body("File uploaded successfully")
}



#[tokio::main]
async fn main() {

    /* 로깅 시작 */
    set_global_logger();
    info!("File Sync Program Start");
    
    // let mut storage = HashStorage {
    //     hashes: HashMap::new(),
    // };
    
    // let mut storage = match HashStorage::load(Path::new("./hash_storage/hash_value.json")) {
    //     Ok(storage) => storage,
    //     Err(e) => panic!("{:?}", e)
    // };
    
    // //.unwrap();

    // println!("test");

    // // 파일 해시 업데이트
    // storage.update_hash("./file3.txt".to_string(), "12345".to_string().into());
    // storage.update_hash("./file6.txt".to_string(), "787788787".to_string().into());
    // storage.update_hash("./file13.txt".to_string(), "testest".to_string().into());
    
    // // 해시값 저장
    // storage.save(Path::new("./hash_storage/hash_value.json")).unwrap();

    // 해시값 로드
    //let loaded_storage = HashStorage::load(Path::new("./hash_storage/hash_value.json")).unwrap();
    //println!("Loaded hashes: {:?}", loaded_storage.hashes);
    

    /* 종속 서비스 호출 */
    let config_req_service = Arc::new(ConfigRequestServicePub::new());
    let watch_service = Arc::new(WatchServicePub::new());

    /* 메인핸들러 호출 */
    let main_handler = MainHandler::new(config_req_service, watch_service);
    
    /* 메인 함수 */
    main_handler.task_main().await;
    
    //let config = read_toml_from_file::<Configs>("./Config.toml").unwrap();
    //println!("{:?}", config);
    
    //test().await.unwrap();
    
    
    
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
    
    // for elem in changes {
    //     println!("{:?}", elem);
    // }
    
    /* ============== HOT Watch ============== */
    // let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
    
    // //let file = "./file_test/master.txt";
    // let files = vec!["./file_test/master2.txt", "./file_test/slave.txt"];
    // //let mut last_hash = hash_file(Path::new(file)).unwrap();
    
    // let (tx, rx) = channel();

    // let value: Result<(), SendError<()>> = tx.send(());
    
    // for file in files.iter() {
         
    //     let file_path = file.to_string();
        
    //     println!("{}", file);

    //     hotwatch.watch(file_path, move |event: Event| {
            
    //         if let WatchEventKind::Modify(_) = event.kind {
    //             value.expect("Failed to send event");
    //             println!("{:?} changed!", event.paths[0]);
    //         }
            
    //     }).expect("failed to watch file!");
    // }
    
        
    // loop {
        
    //     rx.recv().unwrap();
        
    //     //println!("test");
    //     //let cur_hash = hash_file(Path::new(file)).unwrap();

    //     /* 파일 해시값이 다른경우 수정으로 봄 */
    //     // if cur_hash != last_hash {
    //     //     println!("Detected a change!");
    //     //     last_hash = cur_hash;
    //     // }
        
    //     // 필요에 따라 추가 로직 실행
        
    // }


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