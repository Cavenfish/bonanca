use anyhow::Result;
use bip39::{Language, Mnemonic};
use ed25519_dalek_bip32::{DerivationPath, ExtendedSigningKey};
use solana_sdk::signer::keypair::Keypair;

use super::traits::{ChildKey, HDWallet};

pub struct Solana {
    pub coin: u32,
    pub seed: [u8; 64],
    pub master_key: ExtendedSigningKey,
}

impl HDWallet for Solana {
    fn new(language: Language, word_count: usize) -> Self {
        let mut rng = bip39::rand::thread_rng();
        let mnemonic = Mnemonic::generate_in_with(&mut rng, language, word_count)
            .expect("Failed to generate mnemonic");

        let seed = mnemonic.to_seed_normalized("");
        let master_key = ExtendedSigningKey::from_seed(&seed).expect("Failed to create master key");

        Self {
            coin: 501,
            seed: seed,
            master_key: master_key,
        }
    }

    fn derive_child_key(&self, child: u32) -> Result<ChildKey> {
        let derivation_path: DerivationPath =
            format!("m/44'/{}'/{}'/0'/0'", self.coin, child).parse()?;

        let child_key = self.master_key.derive(&derivation_path)?;
        let secret_key = child_key.signing_key;
        let keypair = Keypair::new_from_array(secret_key.to_bytes());

        Ok(ChildKey::Sol(keypair))
    }
}
