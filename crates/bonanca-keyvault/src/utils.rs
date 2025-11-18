use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use anyhow::{Result, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use hex::ToHex;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use solana_sdk::signer::Signer;

use crate::{
    hd_keys::{ChildKey, HDkeys},
    keyvault::{ChainKeys, KdfParams},
};

pub fn verify_password(mac: &str, password: &str) -> bool {
    let hash = PasswordHash::new(mac).expect("Failed to parse MAC");
    let argon2 = Argon2::default();

    argon2.verify_password(password.as_bytes(), &hash).is_ok()
}

pub fn hash_password(password: &str, salt: &SaltString) -> Result<String> {
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), salt)
        .expect("Failed to hash password")
        .to_string();

    Ok(password_hash)
}

pub fn encrypt_seed(
    seed: [u8; 64],
    password: &str,
    nonce: &str,
    kdf_params: &KdfParams,
) -> Result<String> {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        kdf_params.salt.as_bytes(),
        kdf_params.n,
        &mut key,
    );

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let noncebytes = hex::decode(nonce)?;
    let non = GenericArray::from_slice(&noncebytes);

    let ciphertext = cipher
        .encrypt(&non, seed.as_ref())
        .expect("Failed to encrypt master key");

    let text = ciphertext.encode_hex();

    Ok(text)
}

pub fn decrypt_seed(
    cipher_text: &str,
    password: &str,
    nonce: &str,
    kdf_params: &KdfParams,
) -> Result<[u8; 64]> {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        kdf_params.salt.as_bytes(),
        kdf_params.n,
        &mut key,
    );

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let noncebytes = hex::decode(nonce)?;
    let non = GenericArray::from_slice(&noncebytes);

    let ciphertext = hex::decode(cipher_text)?;
    let vec_seed = cipher
        .decrypt(&non, &*ciphertext)
        .expect("Failed to decrypt master key");

    let seed: [u8; 64] = vec_seed.as_slice().try_into()?;

    Ok(seed)
}

fn get_child_pubkey(hd_key: &HDkeys, chain: &str, child: u32) -> Result<String> {
    let child = hd_key.get_child_key(chain, child)?;

    let addy = match child {
        ChildKey::Sol(kp) => kp.pubkey().to_string(),
        ChildKey::Evm(sig) => sig.address().to_string(),
    };

    Ok(addy)
}

pub fn create_chain_keys(hd_key: &HDkeys) -> Result<Vec<ChainKeys>> {
    let sol_addy = get_child_pubkey(&hd_key, "Solana", 0)?;
    let evm_addy = get_child_pubkey(&hd_key, "EVM", 0)?;

    let sol_keys = ChainKeys {
        chain: "Solana".to_string(),
        public_keys: vec![sol_addy],
    };

    let evm_keys = ChainKeys {
        chain: "EVM".to_string(),
        public_keys: vec![evm_addy],
    };

    let chain_keys = vec![sol_keys, evm_keys];

    Ok(chain_keys)
}
