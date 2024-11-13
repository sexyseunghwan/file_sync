use crate::common::*;

use crate::model::Configs::*;

use crate::utils_modules::io_utils::*;


#[doc = "Hash Storage를 전역적으로 사용하기 위함."]
static HASH_STORAGE_CLIENT: once_lazy<Arc<Mutex<HashStorage>>> = once_lazy::new(|| {
    initialize_hash_storage_clients()
});


#[doc = "Hash Storage 를 초기화해주는 함수"]
pub fn initialize_hash_storage_clients() -> Arc<Mutex<HashStorage>> {

    let config: Configs = match read_toml_from_file::<Configs>("./Config.toml") {
        Ok(config) => config,
        Err(e) => {
            error!("[Error][WatchServicePub->new()] {:?}", e);
            panic!("{:?}", e)
        }
    };
    
    let watch_path = config.server.watch_path.clone();
    let hash_map_path = format!("{}hash_storage\\hash_value.json", watch_path);
    
    let hash_storage = match HashStorage::load(Path::new(&hash_map_path)) {
        Ok(hash_storage) => hash_storage,
        Err(e) => {
            error!("[Error][WatchServicePub->new()] Cannot Create HashStorage: {:?}", e);
            panic!("{:?}", e)
        }
    };
    
    Arc::new(Mutex::new(hash_storage))
}


#[doc = ""]
pub fn get_hash_storage() -> Arc<Mutex<HashStorage>> {
    let hash_storage= &HASH_STORAGE_CLIENT;
    Arc::clone(hash_storage)
}


#[derive(Serialize, Deserialize, Debug)]
pub struct HashStorage {
    pub hashes: HashMap<String, Vec<u8>>,
    pub dir_path: String
}

impl HashStorage {

    
    #[doc = "Hashmap file을 읽어서 로드해주는 함수"]
    pub fn load(dir_path: &Path) -> Result<Self, anyhow::Error> {
        
        println!("dir_path= {:?}", dir_path);

        let contents = fs::read_to_string(dir_path)?;
        let dir_path_str = dir_path.to_str().ok_or_else(|| anyhow!("[Error][load()]The path cannot be converted into a string."))?;

        let hash_storage: HashStorage = match serde_json::from_str(&contents) {
            Ok(hashes) => hashes,
            Err(e) => {
                warn!("[WARN][load()] No data exists in file 'hash map': {:?}", e);
                let storage = HashStorage { hashes: HashMap::new(), dir_path: dir_path_str.to_string() };
                storage
            }
        };
        
        Ok(hash_storage)
    }
    
    
    #[doc = "해쉬파일에 해쉬값을 저장해주는 함수."]
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
        
        let hash_val: Vec<u8> = self.hashes.get(file_name)
            .unwrap_or(&Vec::new())
            .clone();
        
        hash_val
    }

}