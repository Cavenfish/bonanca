use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::FixedBytes;
use anyhow::Result;
use bip39::{Language, Mnemonic};
use ed25519_dalek_bip32::{DerivationPath, ExtendedSigningKey};
use hdwallet::{ExtendedPrivKey, KeyIndex};
use solana_sdk::signer::keypair::Keypair;

pub enum MasterKey {
    Sol(ExtendedSigningKey),
    Evm(ExtendedPrivKey),
}

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

    pub fn get_master_key(&self, chain: &str) -> Result<MasterKey> {
        match chain {
            "Solana" => {
                let key = ExtendedSigningKey::from_seed(&self.seed)?;
                Ok(MasterKey::Sol(key))
            }
            "EVM" => {
                let key =
                    ExtendedPrivKey::with_seed(&self.seed).expect("Failed to generate master key");
                Ok(MasterKey::Evm(key))
            }
            _ => Err(anyhow::anyhow!("Chain not supported")),
        }
    }
}
