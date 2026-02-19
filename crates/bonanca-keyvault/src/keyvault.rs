use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use aes_gcm::{AeadCore, Aes256Gcm, aead::OsRng};
use anyhow::{Result, anyhow};
use argon2::password_hash::SaltString;
use bip39::Language;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
use serde_json;
use zeroize::Zeroize;

use crate::hd_keys::HDkeys;
use crate::utils::{decrypt_seed, verify_password};

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyVault {
    pub vault: Vault,
    pub chain_keys: HashMap<String, HashMap<u32, String>>,
}

impl KeyVault {
    pub fn new(lang: &str) -> Self {
        let language = match lang {
            "English" => Ok(Language::English),
            "Simplified Chinese" => Ok(Language::SimplifiedChinese),
            "Traditional Chinese" => Ok(Language::TraditionalChinese),
            "French" => Ok(Language::French),
            "Italian" => Ok(Language::Italian),
            "Japanese" => Ok(Language::Japanese),
            "Korean" => Ok(Language::Korean),
            "Spanish" => Ok(Language::Spanish),
            _ => Err(anyhow!("Language not supported")),
        }
        .unwrap();

        let word_count: usize = 24;
        let salt = SaltString::generate(&mut OsRng);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let hd_key = HDkeys::new(language, word_count);

        let mut password = prompt_password("Set Keyvault Password: ").unwrap();

        let key_vault = hd_key.get_keyvault(&salt, &nonce, &password).unwrap();

        password.zeroize();

        key_vault
    }

    pub fn from_mnemonic(mnemonic: &str) -> Self {
        let hd_key = HDkeys::from_mnemonic(mnemonic);

        let salt = SaltString::generate(&mut OsRng);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let mut password = prompt_password("Set Keyvault Password: ").unwrap();

        let key_vault = hd_key.get_keyvault(&salt, &nonce, &password).unwrap();

        password.zeroize();

        key_vault
    }

    pub fn from_seed(seed: [u8; 64]) -> Self {
        let hd_key = HDkeys { seed };

        let salt = SaltString::generate(&mut OsRng);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let mut password = prompt_password("Set Keyvault Password: ").unwrap();

        let key_vault = hd_key.get_keyvault(&salt, &nonce, &password).unwrap();

        password.zeroize();

        key_vault
    }

    pub fn get_seed(&self) -> Result<[u8; 64]> {
        let mut password = prompt_password("Keyvault Password: ")?;

        if !verify_password(&self.vault.mac, &password) {
            println!("Wrong Password");
            panic!();
        };

        let seed = decrypt_seed(
            &self.vault.cipher_text,
            &password,
            &self.vault.cipher_params.nonce,
            &self.vault.kdf_params,
        )?;

        password.zeroize();

        Ok(seed)
    }

    pub fn load(filename: &Path) -> Self {
        let f = File::open(filename).unwrap();
        let rdr = BufReader::new(f);

        serde_json::from_reader(rdr).unwrap()
    }

    pub fn write(&self, fname: &Path) {
        let file = File::create(fname).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self).unwrap();
    }

    pub fn add_pubkey(&mut self, chain: &str, child: u32, pubkey: &str) {
        let chain_keys = self.chain_keys.get_mut(chain).unwrap();

        chain_keys.insert(child, pubkey.to_string()).unwrap();
    }

    pub fn decrypt_vault(&self) -> Result<HDkeys> {
        let mut password = prompt_password("Keyvault Password: ")?;

        if !verify_password(&self.vault.mac, &password) {
            println!("Wrong Password");
            panic!();
        };

        let seed = decrypt_seed(
            &self.vault.cipher_text,
            &password,
            &self.vault.cipher_params.nonce,
            &self.vault.kdf_params,
        )?;

        password.zeroize();

        Ok(HDkeys { seed })
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
