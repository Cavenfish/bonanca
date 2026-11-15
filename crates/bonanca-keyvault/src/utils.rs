//! Cryptographic helpers backing [`crate::keyvault`].
//!
//! The seed is encrypted with AES-256-GCM under a key derived with
//! PBKDF2-HMAC-SHA256. The GCM tag is the only integrity check and also serves
//! as the password check.

use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use anyhow::{Context, Result};
use hex::ToHex;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::keyvault::KdfParams;

/// Encrypts the 64-byte `seed` with AES-256-GCM and returns it hex-encoded.
///
/// The 32-byte AES key is derived from `password` and the salt/iteration count
/// in `kdf_params` using PBKDF2-HMAC-SHA256; `nonce` is the hex-encoded GCM
/// nonce.
pub(crate) fn encrypt_seed(
    seed: &Zeroizing<[u8; 64]>,
    password: &str,
    nonce: &str,
    kdf_params: &KdfParams,
) -> Result<String> {
    let mut key = Zeroizing::new([0u8; 32]);
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        kdf_params.salt.as_bytes(),
        kdf_params.n,
        &mut *key,
    );

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&*key));
    let noncebytes = hex::decode(nonce).context("Failed to decode nonce")?;
    let non = GenericArray::from_slice(&noncebytes);

    let ciphertext = cipher
        .encrypt(non, seed.as_ref())
        .map_err(|e| anyhow::Error::msg(e.to_string()))
        .context("Failed to encrypt master key")?;

    let text = ciphertext.encode_hex();

    Ok(text)
}

/// Decrypts a hex-encoded seed produced by [`encrypt_seed`].
///
/// The key is re-derived from `password` and `kdf_params`; `nonce` must be the
/// same hex-encoded nonce used during encryption. The returned 64-byte seed is
/// zeroized on drop.
pub(crate) fn decrypt_seed(
    cipher_text: &str,
    password: &str,
    nonce: &str,
    kdf_params: &KdfParams,
) -> Result<Zeroizing<[u8; 64]>> {
    let mut key = Zeroizing::new([0u8; 32]);
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        kdf_params.salt.as_bytes(),
        kdf_params.n,
        &mut *key,
    );

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&*key));
    let noncebytes = hex::decode(nonce).context("Failed to decode nonce")?;
    let non = GenericArray::from_slice(&noncebytes);

    let ciphertext = hex::decode(cipher_text)?;
    let vec_seed = Zeroizing::new(
        cipher
            .decrypt(non, &*ciphertext)
            .map_err(|e| anyhow::Error::msg(e.to_string()))
            .context("Failed to decrypt master key")?,
    );

    let seed = Zeroizing::new(vec_seed.as_slice().try_into()?);

    Ok(seed)
}
