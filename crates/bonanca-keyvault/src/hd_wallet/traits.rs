use anyhow::Result;
use bip39::Language;
use ed25519_dalek_bip32::ExtendedSigningKey;

pub trait HDWallet {
    fn new(language: Language, word_count: usize) -> Self;

    fn derive_child_key(&self, child: u32) -> Result<ExtendedSigningKey>;
}
