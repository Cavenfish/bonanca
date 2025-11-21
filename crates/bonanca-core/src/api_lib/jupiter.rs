use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::deserialize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::VersionedTransaction;
use std::collections::HashMap;

use super::traits::{Exchange, Oracle, SwapTransactionData};
use crate::{holdings::Asset, wallets::traits::Wallet};

pub struct Jupiter {
    pub base_url: String,
    pub api_key: String,
}

impl Jupiter {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self { base_url, api_key }
    }

    pub async fn get_ultra_order(
        &self,
        sell: &str,
        buy: &str,
        amount: u64,
        taker: &str,
    ) -> Result<JupiterUltraOrder> {
        let client = Client::new();

        let url = format!(
            "{}/ultra/v1/order?inputMint={}&outputMint={}&amount={}&taker={}",
            &self.base_url, sell, buy, amount, taker
        );

        let order: JupiterUltraOrder = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<JupiterUltraOrder>()
            .await?;

        Ok(order)
    }

    pub async fn get_price_quote(&self, token: &str) -> Result<HashMap<String, TokenPrice>> {
        let client = Client::new();

        let url = format!("{}/price/v3?ids={}", &self.base_url, token);

        let quote: HashMap<String, TokenPrice> = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<HashMap<String, TokenPrice>>()
            .await?;

        Ok(quote)
    }

    pub async fn get_swap_quote(
        &self,
        sell: &str,
        buy: &str,
        amount: u64,
    ) -> Result<JupiterSwapQuote> {
        let client = Client::new();

        let url = format!(
            "{}/swap/v1/quote?inputMint={}&outputMint={}&amount={}",
            &self.base_url, sell, buy, amount
        );

        let quote: JupiterSwapQuote = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<JupiterSwapQuote>()
            .await?;

        Ok(quote)
    }

    pub async fn get_swap_order(
        &self,
        pubkey: &str,
        swap_quote: JupiterSwapQuote,
    ) -> Result<SwapOrder> {
        let client = Client::new();

        let url = format!("{}/swap/v1/swap", &self.base_url);

        let swap_data = SwapData {
            user_public_key: pubkey.to_string(),
            quote_response: swap_quote,
        };

        let order: SwapOrder = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&swap_data)
            .send()
            .await?
            .json::<SwapOrder>()
            .await?;

        Ok(order)
    }
}

#[async_trait]
impl Exchange for Jupiter {
    async fn get_swap_data(
        &self,
        wallet: &Box<dyn Wallet + Send + Sync>,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> Result<SwapTransactionData> {
        let taker = wallet.get_pubkey()?;
        let big_amount = wallet.parse_token_amount(amount, sell).await?;
        let swap_quote = self.get_swap_quote(sell, buy, big_amount).await?;

        let swap_order = self.get_swap_order(&taker, swap_quote).await?;

        let swap_tx_bytes = STANDARD
            .decode(swap_order.swap_transaction)
            .expect("Failed to decode base64 transaction");

        let tx: VersionedTransaction = deserialize(&swap_tx_bytes).unwrap();

        Ok(SwapTransactionData::Sol(tx))
    }
}

#[async_trait]
impl Oracle for Jupiter {
    async fn get_token_value(&self, asset: &Asset, amount: f64) -> Result<f64> {
        let quote_map = self.get_price_quote(&asset.address).await?;
        let quote = quote_map.get(&asset.address).unwrap();
        let value = amount * quote.usd_price;

        Ok(value)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPrice {
    pub usd_price: f64,
    pub block_id: u64,
    pub decimals: u8,
    pub price_change_24h: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterUltraOrder {
    pub in_amount: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u32,
    pub price_impact_pct: String,
    pub route_plan: Vec<RoutePlan>,
    pub fee_mint: String,
    pub fee_bps: u32,
    pub platform_fee: PlatformFee,
    pub signature_fee_lamports: u64,
    pub signature_fee_payer: Option<String>,
    pub prioritization_fee_lamports: u64,
    pub prioritization_fee_payer: Option<String>,
    pub rent_fee_lamports: u64,
    pub rent_fee_payer: Option<String>,
    pub transaction: Option<String>,
    pub gasless: bool,
    pub taker: Option<String>,
    pub mode: String,
    pub input_mint: String,
    pub output_mint: String,
    pub swap_type: String,
    pub router: String,
    pub request_id: String,
    pub in_usd_value: f64,
    pub out_usd_value: f64,
    pub price_impact: f64,
    pub swap_usd_value: f64,
    pub total_time: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterSwapQuote {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u32,
    pub platform_fee: Option<String>,
    pub price_impact_pct: String,
    pub route_plan: Vec<RoutePlan>,
    pub context_slot: u64,
    pub time_taken: f64,
    pub swap_usd_value: String,
    pub simpler_route_used: bool,
    pub use_incurred_slippage_for_quoting: Option<String>,
    pub other_route_plans: Option<String>,
    pub loaded_longtail_token: bool,
    pub instruction_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapOrder {
    pub swap_transaction: String,
    pub last_valid_block_height: u32,
    pub prioritization_fee_lamports: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: u32,
    pub bps: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    pub amm_key: String,
    pub label: String,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

#[derive(Debug, Deserialize)]
pub struct PlatformFee {
    #[serde(rename = "feeBps")]
    pub fee_bps: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapData {
    pub user_public_key: String,
    pub quote_response: JupiterSwapQuote,
}
