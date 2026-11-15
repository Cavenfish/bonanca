//! Hierarchical deterministic key generation from a master seed.
//!
//! [`HDkeys`] holds the raw 64-byte BIP-39 seed and can either generate a new
//! mnemonic, be built from an existing one, or derive child private keys for
//! the supported curves.

use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{Context, Result};
use argon2::password_hash::SaltString;
use bip32::XPrv;
use bip39::{Language, Mnemonic};
use ed25519_dalek_bip32::{DerivationPath, ExtendedSigningKey};
use zeroize::Zeroizing;

use crate::keyvault::{CipherParams, KdfParams, KeyVault, Vault};
use crate::utils::{encrypt_seed, hash_password};

/// A master seed from which per-chain keys are derived.
///
/// The seed is kept in a [`Zeroizing`] buffer so it is wiped from memory when
/// dropped.
pub struct HDkeys {
    /// The 64-byte BIP-39 seed, zeroized on drop.
    pub seed: Zeroizing<[u8; 64]>,
}

impl HDkeys {
    /// Generates a new random mnemonic and derives the matching seed.
    ///
    /// `word_count` is the mnemonic length (12, 15, 18, 21 or 24 words).
    /// Returns the keys together with the generated mnemonic, which the caller
    /// is responsible for showing to the user and confirming they backed up.
    pub fn new(language: Language, word_count: usize) -> Result<(Self, Zeroizing<String>)> {
        let mut rng = bip39::rand::thread_rng();
        let mnemonic = Mnemonic::generate_in_with(&mut rng, language, word_count)
            .context("Failed to generate mnemonic")?;

        let seed = Zeroizing::new(mnemonic.to_seed_normalized(""));
        let mnemonic_str = Zeroizing::new(mnemonic.to_string());

        Ok((Self { seed }, mnemonic_str))
    }

    /// Derives the seed from an existing BIP-39 mnemonic phrase.
    ///
    /// Fails if the phrase is not a valid mnemonic in any supported language.
    pub fn from_mnemonic(mnemonic_str: &str) -> Result<Self> {
        let mnemonic = Mnemonic::from_str(mnemonic_str)?;

        let seed = Zeroizing::new(mnemonic.to_seed_normalized(""));

        Ok(Self { seed })
    }

    /// Encrypts the seed into a new [`KeyVault`] protected by `password`.
    ///
    /// The key used for AES-256-GCM encryption is derived from `password` and
    /// the PBKDF2 salt inside the generated [`KdfParams`]; `nonce` is the
    /// hex-encoded GCM nonce and `salt` is used for the Argon2id password MAC.
    /// The returned vault has no chain keys recorded yet.
    pub fn get_keyvault(&self, salt: &SaltString, nonce: &str, password: &str) -> Result<KeyVault> {
        let mac = hash_password(&password, salt)?;

        let kdf_params = KdfParams {
            key_length: 32,
            n: 600_000, // OWASP recommendation
            salt: salt.as_str().to_string(),
        };

        let cipher_text = encrypt_seed(&self.seed, &password, nonce, &kdf_params)?;

        let cipher_params = CipherParams {
            nonce: nonce.to_string(),
        };

        let vault = Vault {
            cipher: "aes256-gcm".to_string(),
            cipher_params,
            cipher_text,
            kdf: "pbkdf2".to_string(),
            kdf_params,
            mac,
            salt: salt.as_str().to_string(),
        };

        let chain_keys: HashMap<String, String> = HashMap::new();

        let key_vault = KeyVault { vault, chain_keys };

        Ok(key_vault)
    }

    /// Derives an Ed25519 (Solana-style) child private key from `path`.
    ///
    /// `path` must be a string such as `m/44'/501'/0'/0'`; it is parsed as a
    /// BIP-32 derivation path.
    pub fn derive_ed25519_child_prvkey(&self, path: String) -> Result<Zeroizing<[u8; 32]>> {
        let master_key = ExtendedSigningKey::from_seed(&*self.seed)?;

        let derivation_path: DerivationPath = path.parse()?;

        let child_key = master_key.derive(&derivation_path)?;
        let secret_key = child_key.signing_key;

        Ok(Zeroizing::new(secret_key.to_bytes()))
    }

    /// Derives a secp256k1 (EVM-style) child private key from `path`.
    ///
    /// `path` must be a string such as `m/44'/60'/0'/0/0`.
    pub fn derive_secp256k1_child_prvkey(&self, path: String) -> Result<Zeroizing<[u8; 32]>> {
        let child_key = XPrv::derive_from_path(&self.seed, &path.parse()?)?;

        Ok(Zeroizing::new(child_key.to_bytes()))
    }
}
