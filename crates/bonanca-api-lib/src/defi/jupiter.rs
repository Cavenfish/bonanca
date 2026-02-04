use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;

pub struct JupiterApi {
    pub base_url: String,
    pub api_key: String,
}

impl JupiterApi {
    pub fn new(api_key: String) -> Self {
        Self {
            base_url: "https://api.jup.ag".to_string(),
            api_key,
        }
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
            .header("x-api-key", &self.api_key)
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
            .header("x-api-key", &self.api_key)
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
            .header("x-api-key", &self.api_key)
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
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&swap_data)
            .send()
            .await?
            .json::<SwapOrder>()
            .await?;

        Ok(order)
    }

    pub async fn post_limit_order(&self, body: JupLimitOrder) -> Result<JupTxn> {
        let client = Client::new();
        let url = format!("{}/trigger/v1/createOrder", &self.base_url);

        let resp = client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .json::<JupTxn>()
            .await?;

        Ok(resp)
    }

    pub async fn get_lendable_tokens(&self) -> Result<Vec<JupiterLendMarket>> {
        let client = Client::new();
        let url = format!("{}/lend/v1/earn/tokens", self.base_url);

        let tokens = client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .send()
            .await?
            .json::<Vec<JupiterLendMarket>>()
            .await?;

        Ok(tokens)
    }

    pub async fn post_deposit(&self, body: JupEarnInput) -> Result<JupTxn> {
        let client = Client::new();
        let url = format!("{}/lend/v1/earn/deposit", self.base_url);

        let resp = client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .json::<JupTxn>()
            .await?;

        Ok(resp)
    }

    pub async fn post_withdraw(&self, body: JupEarnInput) -> Result<JupTxn> {
        let client = Client::new();
        let url = format!("{}/lend/v1/earn/withdraw", self.base_url);

        let resp = client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .json::<JupTxn>()
            .await?;

        Ok(resp)
    }
}

#[derive(Debug, Deserialize)]
pub struct JupTxn {
    pub transaction: String,
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
    pub platform_fee: Option<PlatformFee>,
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
    pub percent: Option<u32>,
    pub bps: Option<u32>,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterLendMarket {
    pub id: u64,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub asset_address: String,
    pub asset: Asset,
    pub total_assets: String,
    pub total_supply: String,
    pub convert_to_shares: String,
    pub convert_to_assets: String,
    pub rewards_rate: String,
    pub supply_rate: String,
    pub total_rate: String,
    pub rebalance_difference: String,
    pub liquidity_supply_data: LiquiditySupplyData,
    pub rewards: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub address: String,
    pub chain_id: String,
    pub name: String,
    pub symbol: String,
    pub ui_symbol: String,
    pub decimals: u8,
    pub logo_url: String,
    pub price: String,
    pub coingecko_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquiditySupplyData {
    pub mode_with_interest: bool,
    pub supply: String,
    pub withdrawal_limit: String,
    pub last_update_timestamp: String,
    pub expand_percent: u64,
    pub expand_duration: String,
    pub base_withdrawal_limit: String,
    pub withdrawable_until_limit: String,
    pub withdrawable: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct JupEarnInput {
    pub asset: String,
    pub signer: String,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JupLimitOrder {
    pub maker: String,
    pub payer: String,
    pub input_mint: String,
    pub output_mint: String,
    pub params: JupLimitParams,
}

#[serde_as]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JupLimitParams {
    #[serde_as(as = "DisplayFromStr")]
    pub making_amount: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub taking_amount: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub expired_at: u64,
}
