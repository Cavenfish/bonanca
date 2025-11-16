use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use bip39::Language;
use solana_sdk::signer::keypair::Keypair;

pub enum ChildKey {
    Sol(Keypair),
    Evm(PrivateKeySigner),
}

pub trait HDWallet {
    fn new(language: Language, word_count: usize) -> Self;

    fn derive_child_key(&self, child: u32) -> Result<ChildKey>;
}
