pub struct KeyVault {
    pub valut: Vault,
    pub chain_keys: Vec<ChainKeys>,
}

pub struct Vault {
    pub cipher: String,
    pub cipher_params: CipherParams,
    pub cipher_text: String,
    pub kdf: String,
    pub kdf_params: KdfParams,
    pub mac: String,
}

pub struct CipherParams {
    pub iv: String,
}

pub struct KdfParams {
    pub dklen: u8,
    pub n: u32,
    pub r: u32,
    pub p: u32,
    pub salt: String,
}

pub struct ChainKeys {
    pub chain: String,
    pub public_keys: Vec<String>,
}
