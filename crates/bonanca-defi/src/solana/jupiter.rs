use std::time::Duration;

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::deserialize;
use bonanca_api_lib::defi::jupiter::{
    JupEarnInput, JupLimitOrder, JupLimitParams, JupiterApi, JupiterLendMarket, JupiterSwapQuote,
};
use bonanca_wallets::wallets::solana::{SolTxnReceipt, SolWallet};
use solana_sdk::transaction::VersionedTransaction;

fn make_txn(encoded_txn: String) -> Result<VersionedTransaction> {
    let swap_tx_bytes = STANDARD
        .decode(encoded_txn)
        .expect("Failed to decode base64 transaction");

    let txn: VersionedTransaction = deserialize(&swap_tx_bytes).unwrap();

    Ok(txn)
}

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

        let txn = make_txn(swap_order.swap_transaction)?;

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

        let txn = make_txn(swap_order.swap_transaction)?;
        let sig = wallet.sign_and_send(txn).await.unwrap();

        Ok(sig)
    }

    pub async fn limit_order(
        &self,
        wallet: &SolWallet,
        sell: &str,
        buy: &str,
        sell_amount: f64,
        buy_amount: f64,
        lifetime: Duration,
    ) -> Result<SolTxnReceipt> {
        let make = wallet.parse_token_amount(sell_amount, sell).await?;
        let take = wallet.parse_token_amount(buy_amount, buy).await?;
        let now = wallet.get_timestamp().await? as u64;

        let body = JupLimitOrder {
            maker: wallet.pubkey.to_string(),
            payer: wallet.pubkey.to_string(),
            input_mint: sell.to_string(),
            output_mint: buy.to_string(),
            params: JupLimitParams {
                making_amount: make,
                taking_amount: take,
                expired_at: now + lifetime.as_secs(),
            },
        };

        let data = self.api.post_limit_order(body).await?;
        let txn = make_txn(data.transaction)?;
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
        let body = JupEarnInput {
            asset: token.to_string(),
            signer: wallet.pubkey.to_string(),
            amount: big_amount,
        };

        let data = self.api.post_deposit(body).await?;
        let txn = make_txn(data.transaction)?;
        let sig = wallet.sign_and_send(txn).await.unwrap();

        Ok(sig)
    }

    pub async fn withdraw(
        &self,
        wallet: &SolWallet,
        token: &str,
        amount: f64,
    ) -> Result<SolTxnReceipt> {
        let big_amount = wallet.parse_token_amount(amount, token).await?;
        let body = JupEarnInput {
            asset: token.to_string(),
            signer: wallet.pubkey.to_string(),
            amount: big_amount,
        };

        let data = self.api.post_withdraw(body).await?;
        let txn = make_txn(data.transaction)?;
        let sig = wallet.sign_and_send(txn).await.unwrap();

        Ok(sig)
    }
}
