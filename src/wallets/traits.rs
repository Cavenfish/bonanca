use anyhow::Result;
use std::path::PathBuf;

pub trait Wallet {
    fn load(keystore: PathBuf, rpc: String) -> Self;

    fn get_pubkey(&self) -> Result<String>;

    async fn balance(&self) -> Result<f64>;

    async fn transfer(&self, to: &str, amount: f64) -> Result<()>;

    async fn token_balance(&self, token: &str) -> Result<f64>;

    async fn transfer_token(&self, token: &str, amount: f64, to: &str) -> Result<()>;
}
