use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::deserialize;
use bonanca_api_lib::defi::jupiter::JupiterApi;
use bonanca_wallets::wallets::solana::SolWallet;
use solana_sdk::transaction::VersionedTransaction;

pub struct Jupiter {
    api: JupiterApi,
}

impl Jupiter {
    pub fn new(api_key: String) -> Self {
        let api = JupiterApi::new(api_key);
        Self { api }
    }

    pub async fn swap(&self, wallet: &SolWallet, sell: &str, buy: &str, amount: f64) -> Result<()> {
        let taker = wallet.get_pubkey()?;
        let big_amount = wallet.parse_token_amount(amount, sell).await?;
        let swap_quote = self.api.get_swap_quote(sell, buy, big_amount).await?;

        let swap_order = self.api.get_swap_order(&taker, swap_quote).await?;

        let swap_tx_bytes = STANDARD
            .decode(swap_order.swap_transaction)
            .expect("Failed to decode base64 transaction");

        let txn: VersionedTransaction = deserialize(&swap_tx_bytes).unwrap();

        let _ = wallet.sign_and_send(txn).await.unwrap();

        Ok(())
    }
}
