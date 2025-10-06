use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use crate::exchanges::traits::SwapData;

use super::{evm::EvmWallet, solana::SolWallet};

#[async_trait]
pub trait Wallet {
    fn get_pubkey(&self) -> Result<String>;

    async fn balance(&self) -> Result<f64>;

    async fn transfer(&self, to: &str, amount: f64) -> Result<()>;

    async fn token_balance(&self, token: &str) -> Result<f64>;

    async fn transfer_token(&self, token: &str, amount: f64, to: &str) -> Result<()>;

    async fn swap(&self, swap_data: SwapData) -> Result<()>;
}
