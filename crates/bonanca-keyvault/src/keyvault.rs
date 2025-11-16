use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyVault {
    pub valut: Vault,
    pub chain_keys: Vec<ChainKeys>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Vault {
    pub cipher: String,
    pub cipher_text: String,
    pub kdf: String,
    pub salt: String,
    pub mac: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChainKeys {
    pub chain: String,
    pub public_keys: Vec<String>,
}
