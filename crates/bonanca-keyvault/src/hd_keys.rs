//! Hierarchical deterministic key generation from a master seed.
//!
//! [`HDkeys`] holds the raw 64-byte BIP-39 seed and can either generate a new
//! mnemonic, be built from an existing one, or derive child private keys for
//! the supported curves.

use std::{collections::HashMap, str::FromStr};

use bip32::XPrv;
use bip39::{Language, Mnemonic};
use ed25519_dalek_bip32::{DerivationPath, ExtendedSigningKey};
use zeroize::Zeroizing;

use crate::{
    KeyVaultErrors,
    keyvault::{CipherParams, KdfParams, KeyVault, Vault},
    utils::encrypt_seed,
};

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
    ///
    /// # Errors
    ///
    /// Returns [`KeyVaultErrors::InvalidWordCount`] if `word_count` is not a
    /// supported mnemonic length, or [`KeyVaultErrors::InvalidMnemonic`] for
    /// any other BIP-39 generation failure.
    pub fn new(
        language: Language,
        word_count: usize,
    ) -> Result<(Self, Zeroizing<String>), KeyVaultErrors> {
        let mut rng = bip39::rand::thread_rng();
        let mnemonic = Mnemonic::generate_in_with(&mut rng, language, word_count).map_err(
            |err| match err {
                bip39::Error::BadWordCount(count) => KeyVaultErrors::InvalidWordCount(count),
                err => KeyVaultErrors::InvalidMnemonic(err),
            },
        )?;

        let seed = Zeroizing::new(mnemonic.to_seed_normalized(""));
        let mnemonic_str = Zeroizing::new(mnemonic.to_string());

        Ok((Self { seed }, mnemonic_str))
    }

    /// Derives the seed from an existing BIP-39 mnemonic phrase.
    ///
    /// Fails if the phrase is not a valid mnemonic in any supported language.
    ///
    /// # Errors
    ///
    /// Returns [`KeyVaultErrors::InvalidMnemonic`] if the phrase has an invalid
    /// word count, an unknown word, or a bad checksum.
    pub fn from_mnemonic(mnemonic_str: &str) -> Result<Self, KeyVaultErrors> {
        let mnemonic = Mnemonic::from_str(mnemonic_str).map_err(KeyVaultErrors::InvalidMnemonic)?;

        let seed = Zeroizing::new(mnemonic.to_seed_normalized(""));

        Ok(Self { seed })
    }

    /// Encrypts the seed into a new [`KeyVault`] protected by `password`.
    ///
    /// The key used for AES-256-GCM encryption is derived from `password` and
    /// the PBKDF2 salt stored in the generated [`KdfParams`]; `salt` is the
    /// salt as a string and `nonce` is the hex-encoded GCM nonce. Decrypting the
    /// ciphertext is what checks the password. The returned vault has no
    /// chain keys recorded yet.
    ///
    /// # Errors
    ///
    /// Returns [`KeyVaultErrors::InvalidHex`] or
    /// [`KeyVaultErrors::InvalidNonceLength`] if `nonce` is malformed, or
    /// [`KeyVaultErrors::EncryptionFailed`] if AES-GCM encryption fails.
    pub fn get_keyvault(
        &self,
        salt: String,
        nonce: &str,
        password: &str,
    ) -> Result<KeyVault, KeyVaultErrors> {
        let kdf_params = KdfParams {
            key_length: 32,
            n: 600_000, // OWASP recommendation
            salt,
        };

        let cipher_text = encrypt_seed(&self.seed, &password, nonce, &kdf_params)?;

        let cipher_params = CipherParams { nonce: nonce.to_string() };

        let vault = Vault {
            cipher: "aes256-gcm".to_string(),
            cipher_params,
            cipher_text,
            kdf: "pbkdf2".to_string(),
            kdf_params,
        };

        let chain_keys: HashMap<String, String> = HashMap::new();

        let key_vault = KeyVault { vault, chain_keys };

        Ok(key_vault)
    }

    /// Derives an Ed25519 (Solana-style) child private key from `path`.
    ///
    /// `path` must be a string such as `m/44'/501'/0'/0'`; it is parsed as a
    /// BIP-32 derivation path. Ed25519 derivation requires hardened child
    /// indexes.
    ///
    /// # Errors
    ///
    /// Returns [`KeyVaultErrors::InvalidEd25519Path`] if `path` cannot be
    /// parsed, or [`KeyVaultErrors::Ed25519DerivationFailed`] if the master key
    /// or child key cannot be derived (for example, a non-hardened index).
    pub fn derive_ed25519_child_prvkey(
        &self,
        path: String,
    ) -> Result<Zeroizing<[u8; 32]>, KeyVaultErrors> {
        let master_key = ExtendedSigningKey::from_seed(&*self.seed).map_err(|source| {
            KeyVaultErrors::Ed25519DerivationFailed { path: path.clone(), source }
        })?;

        let derivation_path: DerivationPath = path
            .parse()
            .map_err(|source| KeyVaultErrors::InvalidEd25519Path { path: path.clone(), source })?;

        let child_key = master_key.derive(&derivation_path).map_err(|source| {
            KeyVaultErrors::Ed25519DerivationFailed { path: path.clone(), source }
        })?;
        let secret_key = child_key.signing_key;

        Ok(Zeroizing::new(secret_key.to_bytes()))
    }

    /// Derives a secp256k1 (EVM-style) child private key from `path`.
    ///
    /// `path` must be a string such as `m/44'/60'/0'/0/0`.
    ///
    /// # Errors
    ///
    /// Returns [`KeyVaultErrors::InvalidSecp256k1Path`] if `path` cannot be
    /// parsed, or [`KeyVaultErrors::Secp256k1DerivationFailed`] if the child key
    /// cannot be derived (for example, the maximum depth is exceeded).
    pub fn derive_secp256k1_child_prvkey(
        &self,
        path: String,
    ) -> Result<Zeroizing<[u8; 32]>, KeyVaultErrors> {
        let derivation_path = path.parse::<bip32::DerivationPath>().map_err(|source| {
            KeyVaultErrors::InvalidSecp256k1Path { path: path.clone(), source }
        })?;

        let child_key = XPrv::derive_from_path(&self.seed, &derivation_path)
            .map_err(|source| KeyVaultErrors::Secp256k1DerivationFailed { path, source })?;

        Ok(Zeroizing::new(child_key.to_bytes()))
    }
}
