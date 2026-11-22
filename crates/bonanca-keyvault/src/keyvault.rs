//! The on-disk keyvault format and its password-protected lifecycle.
//!
//! A [`KeyVault`] bundles an encrypted master seed ([`Vault`]) with a map of
//! already-derived public keys, and is serialized to JSON. Creating a vault (or
//! decrypting it) prompts for the password interactively via `rpassword`.

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Write},
    path::Path,
};

use aes_gcm::{
    AeadCore, Aes256Gcm,
    aead::{OsRng, rand_core::RngCore},
};
use bip39::Language;
use serde::{Deserialize, Serialize};
use serde_json;
use tempfile::NamedTempFile;
use zeroize::Zeroizing;

use crate::{KeyVaultErrors, hd_keys::HDkeys, utils::decrypt_seed};

/// Generates a random 16-byte salt string
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
    /// languages return an error. `password` is the encryption password.
    /// Returns the vault and mneomonic.
    ///
    /// # Errors
    ///
    /// Returns [`KeyVaultErrors::UnsupportedLanguage`] if `lang` is not one of
    /// the supported names. Generation and encryption errors from
    /// [`HDkeys::new`] and [`HDkeys::get_keyvault`] are propagated unchanged.
    pub fn new(lang: &str, password: &str) -> Result<(Self, Zeroizing<String>), KeyVaultErrors> {
        let language = match lang {
            "English" => Language::English,
            "Simplified Chinese" => Language::SimplifiedChinese,
            "Traditional Chinese" => Language::TraditionalChinese,
            "French" => Language::French,
            "Italian" => Language::Italian,
            "Japanese" => Language::Japanese,
            "Korean" => Language::Korean,
            "Spanish" => Language::Spanish,
            _ => return Err(KeyVaultErrors::UnsupportedLanguage(lang.to_string())),
        };

        let word_count: usize = 24;
        let salt = get_salt_string();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let (hd_key, mneomonic) = HDkeys::new(language, word_count)?;

        let key_vault = hd_key.get_keyvault(salt, &nonce, &password)?;

        Ok((key_vault, mneomonic))
    }

    /// Builds and encrypts a vault from an existing mnemonic phrase.
    ///
    /// Takes in mnemonic and password.
    ///
    /// # Errors
    ///
    /// Propagates [`KeyVaultErrors::InvalidMnemonic`] from
    /// [`HDkeys::from_mnemonic`] and encryption failures from
    /// [`HDkeys::get_keyvault`].
    pub fn from_mnemonic(mnemonic: &str, password: &str) -> Result<Self, KeyVaultErrors> {
        let hd_key = HDkeys::from_mnemonic(mnemonic)?;

        let salt = get_salt_string();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let key_vault = hd_key.get_keyvault(salt, &nonce, &password)?;

        Ok(key_vault)
    }

    /// Builds and encrypts a vault from a raw 64-byte seed.
    ///
    /// Takes in a 64-byte seed and the password
    ///
    /// # Errors
    ///
    /// Propagates the errors from [`HDkeys::get_keyvault`], including
    /// [`KeyVaultErrors::EncryptionFailed`] if encryption fails.
    pub fn from_seed(seed: [u8; 64], password: &str) -> Result<Self, KeyVaultErrors> {
        let hd_key = HDkeys { seed: Zeroizing::new(seed) };

        let salt = get_salt_string();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let nonce = hex::encode(nonce);

        let key_vault = hd_key.get_keyvault(salt, &nonce, &password)?;

        Ok(key_vault)
    }

    /// Retrieves the decrypted seed.
    ///
    /// Fails if the password is wrong or the ciphertext was tampered with;
    /// AES-GCM authentication cannot tell the two apart.
    ///
    /// # Errors
    ///
    /// Returns [`KeyVaultErrors::UnsupportedCipher`],
    /// [`KeyVaultErrors::UnsupportedKdf`], or
    /// [`KeyVaultErrors::InvalidKdfParams`] if the vault header describes
    /// algorithms or parameters this crate cannot use. Otherwise propagates
    /// the errors from the internal decryption helper, including
    /// [`KeyVaultErrors::DecryptionFailed`].
    pub fn get_seed(&self, password: &str) -> Result<Zeroizing<[u8; 64]>, KeyVaultErrors> {
        self.vault.validate()?;

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
    ///
    /// # Errors
    ///
    /// Returns [`KeyVaultErrors::ReadFailed`] if the file cannot be opened, or
    /// [`KeyVaultErrors::InvalidJson`] if its contents are not a valid
    /// [`KeyVault`]. Algorithm and parameter checks happen on first decryption,
    /// not here.
    pub fn load(filename: &Path) -> Result<Self, KeyVaultErrors> {
        let f = File::open(filename).map_err(|source| KeyVaultErrors::ReadFailed {
            path: filename.to_path_buf(),
            source,
        })?;
        let rdr = BufReader::new(f);

        serde_json::from_reader(rdr).map_err(KeyVaultErrors::InvalidJson)
    }

    /// Serializes the vault as pretty-printed JSON to `fname`.
    ///
    /// The content is first written to a temporary file in the same directory
    /// and then renamed, so an existing vault is never left partially written.
    ///
    /// # Errors
    ///
    /// Returns [`KeyVaultErrors::InvalidJson`] if serialization fails, or
    /// [`KeyVaultErrors::WriteFailed`] with `fname` if the temporary file
    /// cannot be created, written, or renamed into place.
    pub fn write(&self, fname: &Path) -> Result<(), KeyVaultErrors> {
        let mut tmpfile = NamedTempFile::new()
            .map_err(|source| KeyVaultErrors::WriteFailed { path: fname.to_path_buf(), source })?;

        let content = serde_json::to_string_pretty(&self).map_err(KeyVaultErrors::InvalidJson)?;

        tmpfile
            .write_all(content.as_bytes())
            .map_err(|source| KeyVaultErrors::WriteFailed { path: fname.to_path_buf(), source })?;

        fs::rename(tmpfile.path(), fname)
            .map_err(|source| KeyVaultErrors::WriteFailed { path: fname.to_path_buf(), source })?;
        Ok(())
    }

    /// Records `pubkey` as the public key for derivation `path`.
    ///
    /// Any existing entry for the same path is overwritten.
    pub fn add_pubkey(&mut self, path: &str, pubkey: &str) {
        self.chain_keys.insert(path.to_string(), pubkey.to_string());
    }

    /// Retrieves the public key previously recorded for `path`.
    ///
    /// # Errors
    ///
    /// Returns [`KeyVaultErrors::MissingPublicKey`] if no key has been recorded
    /// for `path`.
    pub fn get_pubkey(&self, path: &str) -> Result<&str, KeyVaultErrors> {
        self.chain_keys
            .get(path)
            .map(String::as_str)
            .ok_or_else(|| KeyVaultErrors::MissingPublicKey(path.to_string()))
    }

    /// Retrieves the seed wrapped in [`HDkeys`].
    ///
    /// Fails if the password is wrong or the ciphertext was tampered with;
    /// AES-GCM authentication cannot tell the two apart.
    ///
    /// # Errors
    ///
    /// Same as [`KeyVault::get_seed`]: vault header validation errors followed
    /// by the errors from the internal decryption helper.
    pub fn decrypt_vault(&self, password: &str) -> Result<HDkeys, KeyVaultErrors> {
        self.vault.validate()?;

        let seed = decrypt_seed(
            &self.vault.cipher_text,
            &password,
            &self.vault.cipher_params.nonce,
            &self.vault.kdf_params,
        )?;

        Ok(HDkeys { seed })
    }
}

impl Vault {
    /// Rejects vaults whose header names unsupported algorithms or unusable
    /// KDF parameters before any key derivation or decryption is attempted.
    fn validate(&self) -> Result<(), KeyVaultErrors> {
        if self.cipher != "aes256-gcm" {
            return Err(KeyVaultErrors::UnsupportedCipher(self.cipher.clone()));
        }
        if self.kdf != "pbkdf2" {
            return Err(KeyVaultErrors::UnsupportedKdf(self.kdf.clone()));
        }
        if self.kdf_params.key_length != 32 {
            return Err(KeyVaultErrors::InvalidKdfParams(
                "key_length must be 32 bytes for AES-256",
            ));
        }
        if self.kdf_params.n == 0 {
            return Err(KeyVaultErrors::InvalidKdfParams(
                "PBKDF2 iteration count must be greater than zero",
            ));
        }
        if self.kdf_params.salt.is_empty() {
            return Err(KeyVaultErrors::InvalidKdfParams("PBKDF2 salt must not be empty"));
        }
        Ok(())
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
