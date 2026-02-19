use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;
use argon2::password_hash::SaltString;
use bip32::XPrv;
use bip39::{Language, Mnemonic};
use ed25519_dalek_bip32::{DerivationPath, ExtendedSigningKey};
use zeroize::ZeroizeOnDrop;

use crate::keyvault::{CipherParams, KdfParams, KeyVault, Vault};
use crate::utils::{encrypt_seed, hash_password};

#[derive(ZeroizeOnDrop)]
pub struct HDkeys {
    pub seed: [u8; 64],
}

impl HDkeys {
    pub fn new(language: Language, word_count: usize) -> Self {
        let mut rng = bip39::rand::thread_rng();
        let mnemonic = Mnemonic::generate_in_with(&mut rng, language, word_count)
            .expect("Failed to generate mnemonic");

        // TODO: Ensure this is safest way of 1-time
        // display of mnemonic
        println!("Your mnemonic is:\n");
        println!("\t{}\n", mnemonic);
        println!("Safely store this offline, then clear the terminal");

        let seed = mnemonic.to_seed_normalized("");

        Self { seed }
    }

    pub fn from_mnemonic(mnemonic_str: &str) -> Self {
        let mnemonic = Mnemonic::from_str(mnemonic_str).unwrap();

        let seed = mnemonic.to_seed_normalized("");

        Self { seed }
    }

    pub fn get_keyvault(&self, salt: &SaltString, nonce: &str, password: &str) -> Result<KeyVault> {
        let mac = hash_password(&password, salt)?;

        let kdf_params = KdfParams {
            key_length: 32,
            n: 600_000, // OWASP recommendation
            salt: salt.as_str().to_string(),
        };

        let cipher_text = encrypt_seed(self.seed, &password, nonce, &kdf_params)?;

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

        let chain_keys = self.create_chain_keys()?;

        let key_vault = KeyVault { vault, chain_keys };

        Ok(key_vault)
    }

    pub fn derive_ed25519_child_prvkey(&self, slip: u64, child: u32) -> Result<[u8; 32]> {
        let master_key = ExtendedSigningKey::from_seed(&self.seed)?;

        let derivation_path: DerivationPath = format!("m/44'/{slip}'/{child}'/0'/0'").parse()?;

        let child_key = master_key.derive(&derivation_path)?;
        let secret_key = child_key.signing_key;

        Ok(secret_key.to_bytes())
    }

    pub fn derive_secp256k1_child_prvkey(&self, slip: u64, child: u32) -> Result<[u8; 32]> {
        let path = format!("m/44'/{slip}'/{child}'/0'/0'");
        let child_key = XPrv::derive_from_path(&self.seed, &path.parse()?)?;

        Ok(child_key.to_bytes())
    }

    fn create_chain_keys(&self) -> Result<HashMap<String, HashMap<u32, String>>> {
        let mut chain_keys: HashMap<String, HashMap<u32, String>> = HashMap::new();
        let empty: HashMap<u32, String> = HashMap::new();

        chain_keys.insert("Solana".to_string(), empty.clone());
        chain_keys.insert("EVM".to_string(), empty);

        Ok(chain_keys)
    }
}
