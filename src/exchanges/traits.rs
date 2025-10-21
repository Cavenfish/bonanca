use alloy::rpc::types::TransactionRequest;
use anyhow::Result;
use solana_sdk::transaction::VersionedTransaction;

use crate::wallets::traits::Wallet;

pub enum SwapTransactionData {
    Sol(VersionedTransaction),
    Evm(TransactionRequest),
}

pub trait Dex {
    async fn get_swap_data(
        &self,
        wallet: &Box<dyn Wallet>,
        sell: &str,
        buy: &str,
        amount: u64,
    ) -> Result<SwapTransactionData>;
}
