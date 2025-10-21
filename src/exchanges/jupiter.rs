use crate::{
    exchanges::traits::{Dex, SwapData},
    wallets::traits::Wallet,
};

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::deserialize;
use reqwest::Client;
use serde::Deserialize;
use solana_sdk::transaction::Transaction;

pub struct Jupiter {
    pub base_url: String,
    pub api_key: String,
}

impl Jupiter {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url: base_url,
            api_key: api_key,
        }
    }

    pub async fn get_ultra_order(
        self,
        sell: &str,
        buy: &str,
        amount: u64,
        taker: &str,
    ) -> Result<UltraOrder> {
        let client = Client::new();

        let url = format!(
            "{}/ultra/v1/order?inputMint={}&outputMint={}&amount={}&taker={}",
            &self.base_url, sell, buy, amount, taker
        );

        let order: UltraOrder = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<UltraOrder>()
            .await?;

        Ok(order)
    }

    pub async fn get_swap_quote(self, sell: &str, buy: &str, amount: u64) -> Result<SwapQuote> {
        let client = Client::new();

        let url = format!(
            "{}/swap/v1/quote?inputMint={}&outputMint={}&amount={}",
            &self.base_url, sell, buy, amount
        );

        let order: SwapQuote = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<SwapQuote>()
            .await?;

        Ok(order)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UltraOrder {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapQuote {
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
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: u32,
    pub bps: u32,
}

#[derive(Debug, Deserialize)]
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
