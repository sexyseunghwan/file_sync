use crate::common::*;

#[doc = "파일을 읽어서 해당 파일의 내용을 해시로 변환해주는 함수"]
pub fn conpute_hash(path: &Path) -> Result<Vec<u8>, anyhow::Error> {

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = Vec::new();

    /* 파일의 메타데이터를 통해 크기를 확인 */
    let metadata = file.metadata()?;
    
    if metadata.len() == 0 {
        file = File::open(path)?;   
    }

    file.read_to_end(&mut buffer)?;
    hasher.update(&buffer);

    Ok(hasher.finalize().to_vec())
}