use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use aes_gcm::{AeadCore, Aes256Gcm, aead::OsRng};
use anyhow::{Context, Result, anyhow, bail};
use argon2::password_hash::SaltString;
use bip39::Language;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
use serde_json;
use tempfile::NamedTempFile;
use zeroize::Zeroizing;

use crate::hd_keys::HDkeys;
use crate::utils::{decrypt_seed, verify_password};

fn ask_password() -> Result<Zeroizing<String>> {
    let password = Zeroizing::new(prompt_password("Set Keyvault Password: ")?);
    let confirm = Zeroizing::new(prompt_password("Confirm Password: ")?);

    if password != confirm {
        bail!("Passwords do not match");
    }

    Ok(password)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyVault {
    pub vault: Vault,
    pub chain_keys: HashMap<String, String>,
}

impl KeyVault {
    pub fn new(lang: &str) -> Result<(Self, Zeroizing<String>)> {
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
        }?;

        let word_count: usize = 24;
        let salt = SaltString::generate(&mut OsRng);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let (hd_key, mneomonic) = HDkeys::new(language, word_count)?;

        let password = ask_password().context("Failed to get password")?;

        let key_vault = hd_key.get_keyvault(&salt, &nonce, &password)?;

        Ok((key_vault, mneomonic))
    }

    pub fn from_mnemonic(mnemonic: &str) -> Result<Self> {
        let hd_key = HDkeys::from_mnemonic(mnemonic)?;

        let salt = SaltString::generate(&mut OsRng);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let password = ask_password().context("Failed to get password")?;

        let key_vault = hd_key.get_keyvault(&salt, &nonce, &password)?;

        Ok(key_vault)
    }

    pub fn from_seed(seed: [u8; 64]) -> Result<Self> {
        let hd_key = HDkeys {
            seed: Zeroizing::new(seed),
        };

        let salt = SaltString::generate(&mut OsRng);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let password = ask_password().context("Failed to get password")?;

        let key_vault = hd_key.get_keyvault(&salt, &nonce, &password)?;

        Ok(key_vault)
    }

    pub fn get_seed(&self) -> Result<Zeroizing<[u8; 64]>> {
        let password = Zeroizing::new(prompt_password("Keyvault Password: ")?);

        if !verify_password(&self.vault.mac, &password)? {
            bail!("Wrong password");
        };

        let seed = decrypt_seed(
            &self.vault.cipher_text,
            &password,
            &self.vault.cipher_params.nonce,
            &self.vault.kdf_params,
        )?;

        Ok(seed)
    }

    pub fn load(filename: &Path) -> Result<Self> {
        let f = File::open(filename)?;
        let rdr = BufReader::new(f);

        serde_json::from_reader(rdr).context("Failed to parse json file")
    }

    pub fn write(&self, fname: &Path) -> Result<()> {
        let mut tmpfile = NamedTempFile::new()?;

        let content =
            serde_json::to_string_pretty(&self).context("Failed to serialize keyvault")?;

        tmpfile
            .write_all(content.as_bytes())
            .context("Failed to write json to tmp file")?;

        fs::rename(tmpfile.path(), fname)?;
        Ok(())
    }

    pub fn add_pubkey(&mut self, path: &str, pubkey: &str) {
        self.chain_keys.insert(path.to_string(), pubkey.to_string());
    }

    pub fn decrypt_vault(&self) -> Result<HDkeys> {
        let password = Zeroizing::new(prompt_password("Keyvault Password: ")?);

        if !verify_password(&self.vault.mac, &password)? {
            bail!("Wrong password");
        };

        let seed = decrypt_seed(
            &self.vault.cipher_text,
            &password,
            &self.vault.cipher_params.nonce,
            &self.vault.kdf_params,
        )?;

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
