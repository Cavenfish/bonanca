use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use anyhow::{Context, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use hex::ToHex;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::keyvault::KdfParams;

pub(crate) fn verify_password(mac: &str, password: &str) -> Result<bool> {
    let hash = PasswordHash::new(mac)
        .map_err(|e| anyhow::Error::msg(e.to_string()))
        .context("Failed to parse MAC")?;
    let argon2 = Argon2::default();

    Ok(argon2.verify_password(password.as_bytes(), &hash).is_ok())
}

pub(crate) fn hash_password(password: &str, salt: &SaltString) -> Result<String> {
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), salt)
        .expect("Failed to hash password")
        .to_string();

    Ok(password_hash)
}

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
