use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::FixedBytes;
use anyhow::Result;
use bip39::{Language, Mnemonic};
use ed25519_dalek_bip32::{DerivationPath, ExtendedSigningKey};
use hdwallet::{ExtendedPrivKey, KeyIndex};
use solana_sdk::signer::keypair::Keypair;

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

        let seed = mnemonic.to_seed_normalized("");

        Self { seed: seed }
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
}
