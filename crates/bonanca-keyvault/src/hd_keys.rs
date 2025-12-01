use std::str::FromStr;

use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::FixedBytes;
use anyhow::Result;
use argon2::password_hash::SaltString;
use bip39::{Language, Mnemonic};
use ed25519_dalek_bip32::{DerivationPath, ExtendedSigningKey};
use hdwallet::{ExtendedPrivKey, KeyIndex};
use rpassword::prompt_password;
use solana_sdk::signer::{Signer, keypair::Keypair};

use crate::keyvault::{ChainKeys, CipherParams, KdfParams, KeyVault, Vault};
use crate::utils::{encrypt_seed, hash_password};

pub enum ChildKey {
    Sol(Keypair),
    Evm(PrivateKeySigner),
}

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

    pub fn get_keyvault(
        &self,
        salt: &SaltString,
        nonce: &str,
        pass: Option<&str>,
    ) -> Result<KeyVault> {
        let password = match pass {
            Some(txt) => txt.to_string(),
            None => prompt_password("\nKeyvault Password: ")?,
        };

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

    pub fn get_child_key(&self, chain: &str, child: u32) -> Result<ChildKey> {
        match chain {
            "Solana" => {
                let master_key = ExtendedSigningKey::from_seed(&self.seed)?;

                let derivation_path: DerivationPath =
                    format!("m/44'/501'/{}'/0'/0'", child).parse()?;

                let child_key = master_key.derive(&derivation_path)?;
                let secret_key = child_key.signing_key;
                let keypair = Keypair::new_from_array(secret_key.to_bytes());

                Ok(ChildKey::Sol(keypair))
            }
            "EVM" => {
                let master_key =
                    ExtendedPrivKey::with_seed(&self.seed).expect("Failed to generate master key");

                let key_index = KeyIndex::Normal(child);
                let child_key = master_key.derive_private_key(key_index)?;
                let key_bytes = FixedBytes::new(child_key.private_key.secret_bytes());
                let signer = PrivateKeySigner::from_bytes(&key_bytes)?;

                Ok(ChildKey::Evm(signer))
            }
            _ => Err(anyhow::anyhow!("Chain not supported")),
        }
    }

    pub fn get_child_pubkey(&self, chain: &str, child: u32) -> Result<String> {
        let child = self.get_child_key(chain, child)?;

        let addy = match child {
            ChildKey::Sol(kp) => kp.pubkey().to_string(),
            ChildKey::Evm(sig) => sig.address().to_string(),
        };

        Ok(addy)
    }

    fn create_chain_keys(&self) -> Result<Vec<ChainKeys>> {
        let sol_addy = self.get_child_pubkey("Solana", 0)?;
        let evm_addy = self.get_child_pubkey("EVM", 0)?;

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
}
