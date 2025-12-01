use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs::File, io::BufWriter, path::Path};

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyVault {
    pub vault: Vault,
    pub chain_keys: Vec<ChainKeys>,
}

impl KeyVault {
    pub fn write(&self, fname: &Path) {
        let file = File::create(fname).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self).unwrap();
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Vault {
    pub cipher: String,
    pub cipher_params: CipherParams,
    pub cipher_text: String,
    pub kdf: String,
    pub kdf_params: KdfParams,
    pub mac: String,
    pub salt: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KdfParams {
    pub key_length: u8,
    pub n: u32,
    pub salt: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CipherParams {
    pub nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChainKeys {
    pub chain: String,
    pub public_keys: Vec<String>,
}
