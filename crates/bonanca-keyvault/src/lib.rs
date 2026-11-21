//! Hierarchical deterministic (HD) key generation and encrypted seed storage.
//!
//! This crate provides the primitives used by Bonanca to keep a wallet's master
//! seed at rest and to derive per-chain private keys from it:
//!
//! - [`hd_keys::HDkeys`] wraps a BIP-39 seed and derives Ed25519 (Solana) and
//!   secp256k1 (EVM) child keys from derivation paths.
//! - [`keyvault::KeyVault`] is the serializable, encrypted container that is
//!   written to disk and read back by the wallet crates.
//!
//! The seed is encrypted with AES-256-GCM under a key derived from the user's
//! password via PBKDF2. There is no separate password verifier: the GCM
//! authentication tag doubles as the password check, so a wrong password and a
//! tampered ciphertext are deliberately indistinguishable.
//!
//! # Errors
//!
//! Fallible operations return [`KeyVaultErrors`]. Its variants fall into four
//! groups:
//!
//! - **Input errors** — unsupported languages, malformed mnemonics, and
//!   derivation paths that cannot be parsed or used.
//! - **Vault format errors** — unsupported cipher/KDF identifiers, invalid hex,
//!   bad nonce length, or unusable KDF parameters. These are detected before
//!   decryption where possible, so a corrupt vault is not reported as a wrong
//!   password.
//! - **Crypto failures** — [`KeyVaultErrors::EncryptionFailed`] and
//!   [`KeyVaultErrors::DecryptionFailed`]. The latter intentionally covers both
//!   a wrong password and modified ciphertext: the GCM tag cannot tell them
//!   apart, and revealing which one occurred would leak information about the
//!   encrypted seed.
//! - **Storage errors** — file read/write failures (including the affected
//!   path) and JSON serialization/deserialization failures.
//!
//! The enum is `#[non_exhaustive]`, so external matches should include a
//! wildcard arm. Error messages and sources never include the password, seed,
//! or ciphertext.

pub mod hd_keys;
pub mod keyvault;
mod utils;

use std::path::PathBuf;

use thiserror::Error;

/// Errors produced while creating, decrypting, deriving from, or persisting a
/// [`keyvault::KeyVault`].
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum KeyVaultErrors {
    /// The language name passed to [`keyvault::KeyVault::new`] is not one of
    /// the BIP-39 languages this crate maps by name.
    #[error("unsupported mnemonic language: {0}")]
    UnsupportedLanguage(String),

    /// A mnemonic was requested with a word count other than 12, 15, 18, 21,
    /// or 24. Carries the requested count.
    #[error("invalid mnemonic word count: {0} (expected 12, 15, 18, 21 or 24)")]
    InvalidWordCount(usize),

    /// A mnemonic phrase failed BIP-39 parsing, for example because of an
    /// unknown word or an invalid checksum. The BIP-39 error is the source.
    #[error("invalid BIP-39 mnemonic")]
    InvalidMnemonic(#[source] bip39::Error),

    /// A Solana-style derivation path could not be parsed. Carries the path
    /// string and the parse error as source.
    #[error("invalid ed25519 derivation path: {path}")]
    InvalidEd25519Path {
        path: String,
        #[source]
        source: ed25519_dalek_bip32::derivation_path::DerivationPathParseError,
    },

    /// An EVM-style derivation path could not be parsed. Carries the path
    /// string and the BIP-32 error as source.
    #[error("invalid secp256k1 derivation path: {path}")]
    InvalidSecp256k1Path {
        path: String,
        #[source]
        source: bip32::Error,
    },

    /// Master or child key derivation failed on ed25519, for example because a
    /// path contained a non-hardened child index.
    #[error("failed to derive ed25519 child key at {path}")]
    Ed25519DerivationFailed {
        path: String,
        #[source]
        source: ed25519_dalek_bip32::Error,
    },

    /// Child key derivation failed on secp256k1, for example because the
    /// maximum derivation depth was exceeded.
    #[error("failed to derive secp256k1 child key at {path}")]
    Secp256k1DerivationFailed {
        path: String,
        #[source]
        source: bip32::Error,
    },

    /// The vault names a cipher other than `aes256-gcm`. Detected when the
    /// vault is decrypted, not when it is loaded.
    #[error("unsupported cipher: {0}")]
    UnsupportedCipher(String),

    /// The vault names a KDF other than `pbkdf2`. Detected when the vault is
    /// decrypted, not when it is loaded.
    #[error("unsupported kdf: {0}")]
    UnsupportedKdf(String),

    /// A hex-encoded field of the vault is malformed. `field` names the
    /// field (`"nonce"` or `"cipher_text"`); the hex error is the source.
    #[error("invalid hex in {field}")]
    InvalidHex {
        field: &'static str,
        #[source]
        source: hex::FromHexError,
    },

    /// The GCM nonce is not 12 bytes. Checked before constructing the nonce
    /// array, which would otherwise panic.
    #[error("invalid GCM nonce length: expected 12 bytes, got {0}")]
    InvalidNonceLength(usize),

    /// The vault's KDF parameters are unusable: the key length is not 32
    /// bytes, the iteration count is zero, or the salt is empty.
    #[error("invalid kdf params: {0}")]
    InvalidKdfParams(&'static str),

    /// AES-GCM encryption failed while building a vault. The underlying
    /// cause is not preserved.
    #[error("unable to encrypt master seed")]
    EncryptionFailed,

    /// A wrong password and tampered ciphertext are deliberately
    /// indistinguishable: the GCM tag is the only integrity check, so this
    /// variant covers both cases and carries no source error.
    #[error("unable to decrypt master seed: wrong password or corrupted vault")]
    DecryptionFailed,

    /// Decryption succeeded but the plaintext is not a 64-byte seed, meaning
    /// the vault is corrupt. Carries the actual length.
    #[error("decrypted seed has length {len}, expected 64")]
    InvalidSeedLength { len: usize },

    /// The vault file could not be opened. Carries the path and I/O error.
    #[error("failed to read keyvault at {path}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// The vault could not be serialized to the temporary file or renamed
    /// into place. Carries the destination path and I/O error.
    #[error("failed to write keyvault to {path}")]
    WriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// JSON serialization or deserialization of a [`keyvault::KeyVault`]
    /// failed. The `serde_json` error is the source.
    #[error("invalid keyvault JSON")]
    InvalidJson(#[source] serde_json::Error),

    /// No public key has been recorded for the requested derivation path.
    #[error("no public key recorded for derivation path: {0}")]
    MissingPublicKey(String),
}

#[cfg(test)]
mod tests {
    use super::hd_keys::HDkeys;
    use super::keyvault::KeyVault;
    use super::utils::decrypt_seed;
    use zeroize::Zeroizing;

    #[test]
    fn test_encrypt_and_decrypt() {
        let password = "password";
        let mnemonic = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
        let hd_keys = HDkeys::from_mnemonic(mnemonic).unwrap();

        let seed_hex = "b6a6d8921942dd9806607ebc2750416b289adea669198769f2e15ed926c3aa92bf88ece232317b4ea463e84b0fcd3b53577812ee449ccc448eb45e6f544e25b6";
        let seed: [u8; 64] = hex::decode(seed_hex)
            .unwrap()
            .as_slice()
            .try_into()
            .unwrap();
        let seed = Zeroizing::new(seed);

        assert_eq!(hd_keys.seed, seed);
        assert_eq!(hex::encode(*hd_keys.seed), seed_hex);

        let salt = "M6lWvNAGuZBSp9fBGAUEqw".to_string();
        let nonce = "287189f34a1433d2de201d08";

        let keyvault = hd_keys.get_keyvault(salt, nonce, password).unwrap();

        let decrypted_seed = decrypt_seed(
            &keyvault.vault.cipher_text,
            password,
            &keyvault.vault.cipher_params.nonce,
            &keyvault.vault.kdf_params,
        )
        .unwrap();

        assert_eq!(hd_keys.seed, decrypted_seed);
    }

    #[test]
    fn test_decrypt_keyvault() {
        let keyvault: KeyVault = serde_json::from_str(
            r#"
            {
                "vault": {
                    "cipher": "aes256-gcm",
                    "cipher_params": { "nonce": "287189f34a1433d2de201d08" },
                    "cipher_text": "7a34170003c0a7b3ccb75bac28757801a7d9b5e1ff062afa4af5f3c03e7d8982eb1f36ccce87436e42b44ffea6bcf39eba8c15d4e79ee0bf012811fca81ae1e112c0f8ae5d8e43ac8cad1ae961b11207",
                    "kdf": "pbkdf2",
                    "kdf_params": {
                    "key_length": 32,
                    "n": 600000,
                    "salt": "M6lWvNAGuZBSp9fBGAUEqw"
                    },
                    "mac": "$argon2id$v=19$m=19456,t=2,p=1$M6lWvNAGuZBSp9fBGAUEqw$/U5VYPmg3+BQj0ttOyPnOUjH7bP23V9/tgvBpovna/8",
                    "salt": "M6lWvNAGuZBSp9fBGAUEqw"
                },
                "chain_keys": {
                    "PATH": "ADDRESS"
                }
            }
            "#,
        ).unwrap();

        let password = "password";
        let seed_hex = "b6a6d8921942dd9806607ebc2750416b289adea669198769f2e15ed926c3aa92bf88ece232317b4ea463e84b0fcd3b53577812ee449ccc448eb45e6f544e25b6";
        let seed: [u8; 64] = hex::decode(seed_hex)
            .unwrap()
            .as_slice()
            .try_into()
            .unwrap();

        let decrypted_seed = decrypt_seed(
            &keyvault.vault.cipher_text,
            password,
            &keyvault.vault.cipher_params.nonce,
            &keyvault.vault.kdf_params,
        )
        .unwrap();

        assert_eq!(seed, *decrypted_seed);
    }
}
