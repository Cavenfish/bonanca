pub mod hd_keys;
pub mod keyvault;
mod utils;

use aes_gcm::{AeadCore, Aes256Gcm, aead::OsRng};
use anyhow::{Result, anyhow};
use argon2::password_hash::SaltString;
use bip39::Language;
use rpassword::prompt_password;
use serde_json;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use crate::hd_keys::HDkeys;
use crate::keyvault::{CipherParams, KdfParams, KeyVault, Vault};
use crate::utils::{create_chain_keys, decrypt_seed, encrypt_seed, hash_password, verify_password};

pub fn new(filename: &Path, lang: &str) -> Result<()> {
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

    let hd_key = HDkeys::new(language, word_count);
    let password = prompt_password("\nKeyvault Password: ").unwrap();

    let mac = hash_password(&password, &salt)?;

    let kdf_params = KdfParams {
        key_length: 32,
        n: 600_000, // OWASP recommendation
        salt: salt.as_str().to_string(),
    };

    let ciphertext = encrypt_seed(hd_key.seed, &password, &nonce, &kdf_params)?;

    let cipher_params = CipherParams { nonce: nonce };

    let vault = Vault {
        cipher: "aes256-gcm".to_string(),
        cipher_params: cipher_params,
        cipher_text: ciphertext,
        kdf: "pbkdf2".to_string(),
        kdf_params: kdf_params,
        mac: mac,
        salt: salt.as_str().to_string(),
    };

    let chain_keys = create_chain_keys(&hd_key)?;

    let key_vault = KeyVault {
        valut: vault,
        chain_keys: chain_keys,
    };

    let contents = serde_json::to_string(&key_vault)?;

    let mut file = File::create(filename)?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn decrypt_keyvault(file: &Path) -> Result<HDkeys> {
    let f = File::open(file)?;
    let rdr = BufReader::new(f);

    let keyvault: KeyVault = serde_json::from_reader(rdr)?;

    let password = prompt_password("Keyvault Password: ")?;

    if !verify_password(&keyvault.valut.mac, &password) {
        println!("Wrong Password");
        panic!();
    };

    let seed = decrypt_seed(
        &keyvault.valut.cipher_text,
        &password,
        &keyvault.valut.cipher_params.nonce,
        &keyvault.valut.kdf_params,
    )?;

    Ok(HDkeys { seed: seed })
}

pub fn read_keyvault(file: &Path) -> Result<KeyVault> {
    let f = File::open(file)?;
    let rdr = BufReader::new(f);

    let keyvault: KeyVault = serde_json::from_reader(rdr)?;

    Ok(keyvault)
}
