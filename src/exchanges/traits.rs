use alloy::rpc::types::TransactionRequest;
use anyhow::Result;
use solana_sdk::transaction::Transaction;

use crate::wallets::traits::Wallet;

pub enum SwapData {
    Sol(Transaction),
    Evm(TransactionRequest),
}

pub trait Dex {
    async fn get_swap_data(
        &self,
        wallet: &Box<dyn Wallet>,
        sell: &str,
        buy: &str,
        amount: u64,
    ) -> Result<SwapData>;
}
