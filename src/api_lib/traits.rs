use alloy::rpc::types::TransactionRequest;
use anyhow::Result;
use async_trait::async_trait;
use solana_sdk::transaction::VersionedTransaction;

use crate::{
    api_lib::{jupiter::Jupiter, zerox::ZeroX},
    finance_tk::indexes::Asset,
    wallets::traits::Wallet,
};

pub enum SwapTransactionData {
    Sol(VersionedTransaction),
    Evm(TransactionRequest),
}

pub enum AllExchanges {
    Zero(ZeroX),
    Jup(Jupiter),
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
    async fn get_token_value(&self, asset: &Asset, amount: f64) -> Result<f64>;
}
