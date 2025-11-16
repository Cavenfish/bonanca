use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::FixedBytes;
use anyhow::Result;
use bip39::{Language, Mnemonic};
use hdwallet::{ExtendedPrivKey, KeyIndex};

use super::traits::{ChildKey, HDWallet};

pub struct Evm {
    pub coin: u32,
    pub seed: [u8; 64],
    pub master_key: ExtendedPrivKey,
}

impl HDWallet for Evm {
    fn new(language: Language, word_count: usize) -> Self {
        let mut rng = bip39::rand::thread_rng();
        let mnemonic = Mnemonic::generate_in_with(&mut rng, language, word_count)
            .expect("Failed to generate mnemonic");

        let seed = mnemonic.to_seed_normalized("");

        let master_key = ExtendedPrivKey::with_seed(&seed).expect("Failed to generate master key");

        Self {
            coin: 60,
            seed: seed,
            master_key: master_key,
        }
    }

    fn derive_child_key(&self, child: u32) -> Result<ChildKey> {
        let key_index = KeyIndex::Normal(child);
        let child_key = self.master_key.derive_private_key(key_index)?;
        let key_bytes = FixedBytes::new(child_key.private_key.secret_bytes());
        let signer = PrivateKeySigner::from_bytes(&key_bytes)?;

        Ok(ChildKey::Evm(signer))
    }
}
