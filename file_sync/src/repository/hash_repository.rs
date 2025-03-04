use crate::common::*;

use crate::utils_modules::io_utils::*;

use crate::configs::Configs::*;

#[doc = "Hash Storage를 전역적으로 사용하기 위함."]
static HASH_STORAGE_CLIENT: once_lazy<Arc<Mutex<HashStorage>>> =
    once_lazy::new(initialize_hash_storage_clients);

#[doc = "Hash Storage 를 초기화해주는 함수"]
pub fn initialize_hash_storage_clients() -> Arc<Mutex<HashStorage>> {
    let hash_file_path: Option<String>;
    {
        let server_config: RwLockReadGuard<'_, Configs> = match get_config_read() {
            Ok(server_config) => server_config,
            Err(e) => {
                error!("[Error][initialize_hash_storage_clients()] {:?}", e);
                panic!("{:?}", e)
            }
        };

        hash_file_path = server_config.server.hash_storage_path().clone();
    }

    let hash_file: String = match hash_file_path {
        Some(hash_file) => hash_file,
        None => {
            error!("[Error][initialize_hash_storage_clients()] Path 'hash_file' does not exist.");
            panic!("[Error][initialize_hash_storage_clients()] Path 'hash_file' does not exist.");
        }
    };

    //let hash_map_dir: String = format!("{}hash_storage", hash_file);

    let hash_storage: HashStorage = match HashStorage::load(&hash_file) {
        Ok(hash_storage) => hash_storage,
        Err(e) => {
            error!(
                "[Error][WatchServicePub->new()] Cannot Create HashStorage: {:?}",
                e
            );
            panic!("{:?}", e)
        }
    };

    Arc::new(Mutex::new(hash_storage))
}

#[doc = "Hash Storage 를 불러와주는 함수"]
pub fn get_hash_storage() -> Arc<Mutex<HashStorage>> {
    let hash_storage: &once_lazy<Arc<Mutex<HashStorage>>> = &HASH_STORAGE_CLIENT;
    Arc::clone(hash_storage)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HashStorage {
    pub hashes: HashMap<String, Vec<u8>>,
    pub dir_path: String,
}

impl HashStorage {
    #[doc = "Hashmap file을 읽어서 로드해주는 함수"]
    pub fn load(hash_map_dir: &str) -> Result<Self, anyhow::Error> {
    
        /* 디렉토리와 파일이 존재하는지 확인 */
        let dir_path: PathBuf = create_dir_and_file(hash_map_dir, "hash_value.json")?;

        let contents: String = fs::read_to_string(&dir_path)?;
        let dir_path_str: &str = dir_path
            .to_str()
            .ok_or_else(|| anyhow!("[Error][load()]The path cannot be converted into a string."))?;

        let mut hash_storage: HashStorage = match serde_json::from_str(&contents) {
            Ok(hashes) => hashes,
            Err(e) => {
                warn!("[WARN][load()] No data exists in file 'hash map': {:?}", e);
                HashStorage {
                    hashes: HashMap::new(),
                    dir_path: dir_path_str.to_string(),
                }
            }
        };

        /*
            HashMap file 이 존재하는 경우 dir_path 파일이 기존이랑 다를 수 있음.
            그럴경우에는 dir_path 를 update 해줘야 함.
        */
        if dir_path_str != hash_storage.dir_path {
            hash_storage.dir_path = dir_path_str.to_string();
            let contents = serde_json::to_string(&hash_storage)?;
            fs::write(dir_path_str, contents)?;
        }

        Ok(hash_storage)
    }

    #[doc = "해쉬파일에 Heap 메모리 상에 존재하는 해쉬값을 저장해주는 함수."]
    pub fn save(&self) -> Result<(), anyhow::Error> {
        let contents = serde_json::to_string(&self)?;
        fs::write(self.dir_path.clone(), contents)?;
        Ok(())
    }

    #[doc = "해쉬파일에서 해쉬값을 업데이트 해주는 함수"]
    pub fn update_hash(&mut self, file_name: String, hash: Vec<u8>) {
        self.hashes.insert(file_name, hash);
    }

    #[doc = "해시 저장소에서 주어진 파일 이름의 해시 값을 조회."]
    pub fn get_hash(&self, file_name: &str) -> Vec<u8> {
        let hash_val: Vec<u8> = self.hashes.get(file_name).unwrap_or(&Vec::new()).clone();

        hash_val
    }
}
