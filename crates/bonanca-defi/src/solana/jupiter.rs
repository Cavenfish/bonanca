use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::deserialize;
use bonanca_api_lib::defi::jupiter::{
    JupEarnDeposit, JupiterApi, JupiterLendMarket, JupiterSwapQuote,
};
use bonanca_wallets::wallets::solana::{SolTxnReceipt, SolWallet};
use solana_sdk::transaction::VersionedTransaction;

pub struct Jupiter {
    api: JupiterApi,
}

impl Jupiter {
    pub fn new(api_key: String) -> Self {
        let api = JupiterApi::new(api_key);
        Self { api }
    }

    pub async fn get_token_price(&self, token: &str, amount: f64) -> Result<f64> {
        let quote_map = self.api.get_price_quote(token).await?;
        let quote = quote_map.get(token).unwrap();
        let value = amount * quote.usd_price;

        Ok(value)
    }

    pub async fn get_swap_quote(
        &self,
        wallet: &SolWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> Result<JupiterSwapQuote> {
        let big_amount = wallet.parse_token_amount(amount, sell).await?;
        self.api.get_swap_quote(sell, buy, big_amount).await
    }

    pub async fn swap(&self, wallet: &SolWallet, quote: JupiterSwapQuote) -> Result<SolTxnReceipt> {
        let taker = wallet.get_pubkey()?;
        let swap_order = self.api.get_swap_order(&taker, quote).await?;

        let swap_tx_bytes = STANDARD
            .decode(swap_order.swap_transaction)
            .expect("Failed to decode base64 transaction");

        let txn: VersionedTransaction = deserialize(&swap_tx_bytes).unwrap();

        let sig = wallet.sign_and_send(txn).await.unwrap();

        Ok(sig)
    }

    pub async fn quick_swap(
        &self,
        wallet: &SolWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> Result<SolTxnReceipt> {
        let taker = wallet.get_pubkey()?;
        let big_amount = wallet.parse_token_amount(amount, sell).await?;
        let swap_quote = self.api.get_swap_quote(sell, buy, big_amount).await?;

        let swap_order = self.api.get_swap_order(&taker, swap_quote).await?;

        let swap_tx_bytes = STANDARD
            .decode(swap_order.swap_transaction)
            .expect("Failed to decode base64 transaction");

        let txn: VersionedTransaction = deserialize(&swap_tx_bytes).unwrap();

        let sig = wallet.sign_and_send(txn).await.unwrap();

        Ok(sig)
    }

    pub async fn get_lendable_tokens(&self) -> Result<Vec<JupiterLendMarket>> {
        self.api.get_lendable_tokens().await
    }

    pub async fn deposit(
        &self,
        wallet: &SolWallet,
        token: &str,
        amount: f64,
    ) -> Result<SolTxnReceipt> {
        let big_amount = wallet.parse_token_amount(amount, token).await?;
        let body = JupEarnDeposit {
            asset: token.to_string(),
            signer: wallet.pubkey.to_string(),
            amount: big_amount,
        };

        let data = self.api.post_deposit(body).await?;

        let swap_tx_bytes = STANDARD
            .decode(data.transaction)
            .expect("Failed to decode base64 transaction");

        let mut txn: VersionedTransaction = deserialize(&swap_tx_bytes).unwrap();

        let hash = wallet.client.get_latest_blockhash().await?;
        txn.message.set_recent_blockhash(hash);

        let sig = wallet.sign_and_send(txn).await.unwrap();

        Ok(sig)
    }
}
