use alloy::rpc::types::TransactionRequest;
use alloy::signers::{k256::ecdsa::SigningKey, local::LocalSigner};
use anyhow::Result;
use async_trait::async_trait;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::VersionedTransaction;

use super::holdings::Asset;

pub enum SwapTransactionData {
    Sol(VersionedTransaction),
    Evm(TransactionRequest),
}

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

#[async_trait]
pub trait Exchange {
    async fn get_swap_data(
        &self,
        wallet: &Box<dyn Wallet + Send + Sync>,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> Result<SwapTransactionData>;
}

#[async_trait]
pub trait Oracle {
    async fn get_token_value(&self, asset: &Asset, amount: f64, chain: &str) -> Result<f64>;
}

#[async_trait]
pub trait Bank {
    async fn get_pools(&self) -> Result<()>;

    async fn get_user_data(&self) -> Result<()>;

    async fn supply(&self, token: &str, amount: u64) -> Result<()>;

    async fn borrow(&self, token: &str, amount: u64) -> Result<()>;

    async fn repay(&self, token: &str, amount: u64) -> Result<()>;

    async fn withdraw(&self, token: &str, amount: u64) -> Result<()>;
}
