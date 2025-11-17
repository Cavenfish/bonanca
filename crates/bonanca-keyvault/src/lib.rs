mod hd_keys;
pub mod hd_wallet;
pub mod keyvault;
mod utils;

use aes_gcm::{AeadCore, Aes256Gcm, aead::OsRng};
use alloy::hex::ToHexExt;
use anyhow::Result;
use argon2::password_hash::SaltString;
use bip39::Language;
use rpassword::prompt_password;
use serde_json;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use crate::hd_keys::HDkeys;
use crate::keyvault::{CipherParams, KdfParams, KeyVault, Vault};
use crate::utils::{decrypt_seed, encrypt_seed, hash_password, verify_password};

pub fn new() -> Result<()> {
    let language = Language::English;
    let word_count: usize = 24;
    let salt = SaltString::generate(&mut OsRng);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let nonce = hex::encode(nonce);

    let hd_key = HDkeys::new(language, word_count);
    let password = prompt_password("Keyvault Password: ").unwrap();

    let mac = hash_password(&password, &salt)?;

    let kdf_params = KdfParams {
        key_length: 32,
        n: 25_000,
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

    let key_vault = KeyVault {
        valut: vault,
        chain_keys: None,
    };

    let contents = serde_json::to_string(&key_vault)?;

    let mut file = File::create("./keyvault_test.json")?;
    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn decrypt_keyvault(file: &Path) -> Result<[u8; 64]> {
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

    Ok(seed)
}
