//! The on-disk keyvault format and its password-protected lifecycle.
//!
//! A [`KeyVault`] bundles an encrypted master seed ([`Vault`]) with a map of
//! already-derived public keys, and is serialized to JSON. Creating a vault (or
//! decrypting it) prompts for the password interactively via `rpassword`.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use aes_gcm::{
    AeadCore, Aes256Gcm,
    aead::{OsRng, rand_core::RngCore},
};
use anyhow::{Context, Result, anyhow, bail};
use bip39::Language;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
use serde_json;
use tempfile::NamedTempFile;
use zeroize::Zeroizing;

use crate::hd_keys::HDkeys;
use crate::utils::decrypt_seed;

/// Interactively prompts twice for a new vault password and checks they match.
fn ask_password() -> Result<Zeroizing<String>> {
    let password = Zeroizing::new(prompt_password("Set Keyvault Password: ")?);
    let confirm = Zeroizing::new(prompt_password("Confirm Password: ")?);

    if password != confirm {
        bail!("Passwords do not match");
    }

    Ok(password)
}

fn get_salt_string() -> String {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    hex::encode(salt)
}

/// An encrypted master seed plus the public keys derived from it.
///
/// This is the serializable form of a wallet, typically stored as a JSON file
/// and loaded with [`KeyVault::load`].
#[derive(Debug, Deserialize, Serialize)]
pub struct KeyVault {
    /// The encrypted seed and the parameters needed to decrypt it.
    pub vault: Vault,
    /// Mapping from derivation path to the hex-encoded public key at that path.
    pub chain_keys: HashMap<String, String>,
}

impl KeyVault {
    /// Generates a fresh mnemonic and encrypts it into a new vault.
    ///
    /// `lang` is the mnemonic language name (e.g. `"English"`); unsupported
    /// languages return an error. The password is read from stdin. Returns the
    /// vault together with the mnemonic, which must be shown to the user for
    /// backup before it is discarded.
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
        let salt = get_salt_string();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let (hd_key, mneomonic) = HDkeys::new(language, word_count)?;

        let password = ask_password().context("Failed to get password")?;

        let key_vault = hd_key.get_keyvault(salt, &nonce, &password)?;

        Ok((key_vault, mneomonic))
    }

    /// Builds and encrypts a vault from an existing mnemonic phrase.
    ///
    /// The password is read from stdin.
    pub fn from_mnemonic(mnemonic: &str) -> Result<Self> {
        let hd_key = HDkeys::from_mnemonic(mnemonic)?;

        let salt = get_salt_string();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let password = ask_password().context("Failed to get password")?;

        let key_vault = hd_key.get_keyvault(salt, &nonce, &password)?;

        Ok(key_vault)
    }

    /// Builds and encrypts a vault from a raw 64-byte seed.
    ///
    /// The password is read from stdin.
    pub fn from_seed(seed: [u8; 64]) -> Result<Self> {
        let hd_key = HDkeys {
            seed: Zeroizing::new(seed),
        };

        let salt = get_salt_string();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let password = ask_password().context("Failed to get password")?;

        let key_vault = hd_key.get_keyvault(salt, &nonce, &password)?;

        Ok(key_vault)
    }

    /// Prompts for the vault password and returns the decrypted seed.
    ///
    /// Fails if the password is wrong or the ciphertext was tampered with;
    /// AES-GCM authentication cannot tell the two apart.
    pub fn get_seed(&self) -> Result<Zeroizing<[u8; 64]>> {
        let password = Zeroizing::new(prompt_password("Keyvault Password: ")?);

        let seed = decrypt_seed(
            &self.vault.cipher_text,
            &password,
            &self.vault.cipher_params.nonce,
            &self.vault.kdf_params,
        )?;

        Ok(seed)
    }

    /// Reads a vault from the JSON file at `filename`.
    ///
    /// No decryption happens here; use [`KeyVault::get_seed`] or
    /// [`KeyVault::decrypt_vault`] for that.
    pub fn load(filename: &Path) -> Result<Self> {
        let f = File::open(filename)?;
        let rdr = BufReader::new(f);

        serde_json::from_reader(rdr).context("Failed to parse json file")
    }

    /// Serializes the vault as pretty-printed JSON to `fname`.
    ///
    /// The content is first written to a temporary file in the same directory
    /// and then renamed, so an existing vault is never left partially written.
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

    /// Records `pubkey` as the public key for derivation `path`.
    ///
    /// Any existing entry for the same path is overwritten.
    pub fn add_pubkey(&mut self, path: &str, pubkey: &str) {
        self.chain_keys.insert(path.to_string(), pubkey.to_string());
    }

    /// Prompts for the vault password and returns the seed wrapped in [`HDkeys`].
    ///
    /// Fails if the password is wrong or the ciphertext was tampered with;
    /// AES-GCM authentication cannot tell the two apart.
    pub fn decrypt_vault(&self) -> Result<HDkeys> {
        let password = Zeroizing::new(prompt_password("Keyvault Password: ")?);

        let seed = decrypt_seed(
            &self.vault.cipher_text,
            &password,
            &self.vault.cipher_params.nonce,
            &self.vault.kdf_params,
        )?;

        Ok(HDkeys { seed })
    }
}

/// The encrypted master seed and the parameters required to decrypt it.
///
/// `cipher`/`kdf` name the algorithms (currently `"aes256-gcm"` and
/// `"pbkdf2"`); the vault carries no separate password verifier, because the
/// AES-GCM tag already fails on a wrong key.
#[derive(Debug, Deserialize, Serialize)]
pub struct Vault {
    /// Identifier of the symmetric cipher used for `cipher_text`.
    pub cipher: String,
    /// Cipher-specific parameters, such as the GCM nonce.
    pub cipher_params: CipherParams,
    /// Hex-encoded ciphertext of the master seed.
    pub cipher_text: String,
    /// Identifier of the key derivation function used to derive the AES key.
    pub kdf: String,
    /// Key derivation parameters, such as the salt and iteration count.
    pub kdf_params: KdfParams,
}

/// Parameters for the PBKDF2 key derivation function.
#[derive(Debug, Deserialize, Serialize)]
pub struct KdfParams {
    /// Length in bytes of the derived AES key.
    pub key_length: u8,
    /// Number of PBKDF2 iterations.
    pub n: u32,
    /// Salt used for key derivation; its string bytes are fed to PBKDF2 as-is.
    pub salt: String,
}

/// Parameters for the AES-256-GCM cipher.
#[derive(Debug, Deserialize, Serialize)]
pub struct CipherParams {
    /// Hex-encoded GCM nonce; must be unique per encryption.
    pub nonce: String,
}
