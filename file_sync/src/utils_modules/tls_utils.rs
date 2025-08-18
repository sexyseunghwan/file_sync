use crate::common::*;
use crate::configs::configs::*;
use rustls::{
    RootCertStore, ServerConfig, ClientConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
    server::{WebPkiClientVerifier, danger::ClientCertVerifier}
};


#[doc = "HTTPS 통신을 위한 key 정보가 존재하는 디렉토리 정보를 읽어주는 함수"]
fn get_key_directory() -> Result<String, anyhow::Error> {
    let server_config: RwLockReadGuard<'static, Configs> = get_config_read()?;
    Ok(server_config.server.get_key_directory())
}


#[doc = ".crt 파일을 읽어주는 함수"]
fn load_cert_chain(path: &str) -> Result<Vec<CertificateDer<'static>>, anyhow::Error> {
    let mut rd: BufReader<File> = BufReader::new(File::open(path)?);
    let der_vec: Vec<CertificateDer<'static>> = certs(&mut rd).collect::<std::result::Result<_, _>>()?;
    Ok(der_vec)
}


#[doc = ".key 파일을 읽어주는 함수"]
fn load_private_key_pkcs8(path: &str) -> Result<PrivateKeyDer<'static>, anyhow::Error> {
    let mut rd: BufReader<File> = BufReader::new(File::open(path)?);
    let keys: Vec<_> = pkcs8_private_keys(&mut rd).collect::<std::result::Result<Vec<_>, _>>()?;
    let mut keys: Vec<PrivateKeyDer<'static>> = keys.into_iter().map(|k| k.into()).collect();
    let key: PrivateKeyDer<'static> = keys
        .pop()
        .ok_or_else(|| anyhow::anyhow!("[ERROR][load_private_key_pkcs8] no PKCS#8 private key found"))?;
    Ok(key)
}

#[doc = "ca.crt 파일을 읽어주는 함수"]
fn load_client_ca_store(path: &str) -> Result<RootCertStore, anyhow::Error> {
    let mut store: RootCertStore = RootCertStore::empty();
    let ca_der_list: Vec<CertificateDer<'static>> = load_cert_chain(path)?;
    for ca in ca_der_list {
        store.add(ca)?;
    }
    Ok(store)
}


#[doc = "Server 환경의 tls를 적용해주는 함수 -> Slave"]
pub fn create_server_tls_config() -> Result<ServerConfig, anyhow::Error> {
    let key_dir: String = get_key_directory()?;
    
    let server_chain: Vec<CertificateDer<'static>> = load_cert_chain(&format!("{}/server.crt", key_dir))?;
    let server_key: PrivateKeyDer<'static> = load_private_key_pkcs8(&format!("{}/server.key", key_dir))?;
    
    let client_roots: RootCertStore = load_client_ca_store(&format!("{}/ca.crt", key_dir))?;
    let client_verifier: Arc<dyn ClientCertVerifier + 'static> = WebPkiClientVerifier::builder(Arc::new(client_roots))
        .build()
        .map_err(|e| anyhow::anyhow!("[ERROR][create_server_tls_config] Failed to build client verifier: {:?}", e))?;

    let tls_config: ServerConfig = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(server_chain, server_key)
        .map_err(|e| anyhow::anyhow!("[ERROR][create_server_tls_config] Invalid certs/keys: {:?}", e))?;

    Ok(tls_config)
}

#[doc = "Client 환경의 tls를 적용해주는 함수 -> Master"]
pub fn create_client_tls_config() -> Result<ClientConfig, anyhow::Error> {
    let key_dir: String = get_key_directory()?;
    
    let ca_der_list: Vec<CertificateDer<'static>> = load_cert_chain(&format!("{}/ca.crt", key_dir))?;
    let client_chain: Vec<CertificateDer<'static>> = load_cert_chain(&format!("{}/client.crt", key_dir))?;
    let client_key: PrivateKeyDer<'static> = load_private_key_pkcs8(&format!("{}/client.key", key_dir))?;
    
    let mut root_store: RootCertStore = RootCertStore::empty();
    for ca in ca_der_list {
        root_store.add(ca)?;
    }

    let tls_config: ClientConfig = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_client_auth_cert(client_chain, client_key)
        .map_err(|e| anyhow::anyhow!("[ERROR][create_client_tls_config] Failed to configure client auth: {:?}", e))?;

    Ok(tls_config)
}