use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::deserialize;
use bonanca_api_lib::defi::jupiter::Jupiter;
use bonanca_wallets::{TransactionData, Wallet};
use solana_sdk::transaction::VersionedTransaction;

use crate::exchange::Exchange;

#[async_trait]
impl Exchange for Jupiter {
    async fn get_swap_data(
        &self,
        wallet: &Box<dyn Wallet + Send + Sync>,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> Result<TransactionData> {
        let taker = wallet.get_pubkey()?;
        let big_amount = wallet.parse_token_amount(amount, sell).await?;
        let swap_quote = self.get_swap_quote(sell, buy, big_amount).await?;

        let swap_order = self.get_swap_order(&taker, swap_quote).await?;

        let swap_tx_bytes = STANDARD
            .decode(swap_order.swap_transaction)
            .expect("Failed to decode base64 transaction");

        let tx: VersionedTransaction = deserialize(&swap_tx_bytes).unwrap();

        Ok(TransactionData::Sol(tx))
    }
}
