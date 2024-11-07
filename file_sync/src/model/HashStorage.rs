use crate::common::*;

#[derive(Serialize, Deserialize)]
pub struct HashStorage {
    pub hashes: HashMap<String, Vec<u8>>,
}


impl HashStorage {
    
    pub fn load(path: &Path) -> Result<Self, anyhow::Error> {
        let contents = fs::read_to_string(path)?;
        let hashes = serde_json::from_str(&contents)?;
        Ok(HashStorage { hashes })
    }


    pub fn save(&self, path: &Path) -> Result<(), anyhow::Error> {
        let contents = serde_json::to_string(&self)?;
        fs::write(path, contents)?;
        Ok(())
    }


    pub fn update_hash(&mut self, file_name: String, hash: Vec<u8>) {
        self.hashes.insert(file_name, hash);
    }
}