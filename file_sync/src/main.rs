mod common;
use crate::common::*;

mod utils_modules;
use utils_modules::logger_utils::*;

mod handler;

mod model;




#[tokio::main]
async fn main() {

    /* 로깅 시작 */
    set_global_logger();
    info!("File Sync Program Start");
    
    
    /* 메인핸들러 호출 */
    
    
    let (tx, rx) = channel();

    let config = Config::default()
        .with_poll_interval(Duration::from_secs(2));

    // Watcher 인스턴스 생성
    let mut watcher: RecommendedWatcher = Watcher::new(tx, config).unwrap();

    // 관찰할 디렉토리 설정
    watcher.watch(Path::new("./file_test/test.txt"), RecursiveMode::Recursive).unwrap();

    println!("Watching for changes...");

    // 이벤트 수신 대기
    for event in rx {
        println!("Change detected: {:?}", event);
    }
    
    
}