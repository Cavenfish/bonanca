use anyhow::Result;
use std::path::PathBuf;

pub trait Wallet {
    fn load(keystore: PathBuf, rpc: String) -> Self;

    async fn balance(&self) -> Result<f64>;

    async fn transfer(&self, to: &str, amount: f64) -> Result<()>;

    async fn token_balance(&self, token: &str) -> Result<f64>;

    async fn transfer_token(&self, token: &str, amount: f64, to: &str) -> Result<()>;
}

pub trait Dex {
    fn new() -> Self;

    async fn swap(&self, sell: &str, buy: &str, amount: u64) -> Result<()>;
}
