use crate::common::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct HashStorage {
    pub hashes: HashMap<String, Vec<u8>>
}


impl HashStorage {
    
    pub fn load(path: &Path) -> Result<Self, anyhow::Error> {
        
        println!("load");
        
        let contents = fs::read_to_string(path)?;

        println!("what?");
        
        //let test: HashStorage = serde_json::from_str(&contents)?;

        //println!("{:?}", test);


        let hashes: HashStorage = match serde_json::from_str(&contents) {
            Ok(hashes) => hashes,
            Err(e) => {
                warn!("[WARN][load()] No data exists in file 'hash map': {:?}", e);
                let storage = HashStorage { hashes: HashMap::new() };
                storage
                //return Ok(storage)
            }
        };
        
        Ok(hashes)
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