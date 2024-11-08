use crate::common::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct HashStorage {
    pub hashes: HashMap<String, Vec<u8>>
}


impl HashStorage {
    
    #[doc = "Hashmap 파일을 읽어서 로드해주는 함수"]
    pub fn load(path: &Path) -> Result<Self, anyhow::Error> {
        
        let contents = fs::read_to_string(path)?;

        let hashes: HashStorage = match serde_json::from_str(&contents) {
            Ok(hashes) => hashes,
            Err(e) => {
                warn!("[WARN][load()] No data exists in file 'hash map': {:?}", e);
                let storage = HashStorage { hashes: HashMap::new() };
                storage
            }
        };
        
        Ok(hashes)
    }
    
    
    #[doc = "해쉬파일에 해쉬값을 저장해주는 함수."]
    pub fn save(&self, path: &Path) -> Result<(), anyhow::Error> {
        let contents = serde_json::to_string(&self)?;
        fs::write(path, contents)?;
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