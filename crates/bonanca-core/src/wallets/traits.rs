use alloy::signers::{k256::ecdsa::SigningKey, local::LocalSigner};
use anyhow::Result;
use async_trait::async_trait;
use solana_sdk::signature::Keypair;

use crate::api_lib::traits::SwapTransactionData;

pub enum CryptoSigners {
    Evm(LocalSigner<SigningKey>),
    Sol(Keypair),
}

#[async_trait]
pub trait Wallet {
    fn get_pubkey(&self) -> Result<String>;

    fn get_signer(&self) -> Result<CryptoSigners>;

    fn parse_native_amount(&self, amount: f64) -> Result<u64>;

    async fn parse_token_amount(&self, amount: f64, token: &str) -> Result<u64>;

    async fn close(&self, to: &str) -> Result<()>;

    async fn balance(&self) -> Result<f64>;

    async fn transfer(&self, to: &str, amount: f64) -> Result<()>;

    async fn token_balance(&self, token: &str) -> Result<f64>;

    async fn transfer_token(&self, token: &str, amount: f64, to: &str) -> Result<()>;

    async fn transfer_all_tokens(&self, token: &str, to: &str) -> Result<()>;

    async fn check_swap(&self, token: &str, amount: f64, spender: Option<&str>) -> Result<bool>;

    async fn swap(&self, swap_data: SwapTransactionData) -> Result<()>;
}
