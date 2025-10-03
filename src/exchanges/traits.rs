use anyhow::Result;

use crate::wallets::traits::Wallet;

pub trait Dex {
    async fn swap<T: Wallet>(&self, wallet: T, sell: &str, buy: &str, amount: u64) -> Result<()>;
}
