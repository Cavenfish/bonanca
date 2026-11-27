//! Jupiter API client for Solana swaps, pricing, limit orders, and lending.

use std::collections::HashMap;

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

/// Client for the Jupiter Swap, Price, Trigger, and Lend APIs.
pub struct JupiterApi {
    base_url: String,
    api_key: String,
    client: Client,
}

impl JupiterApi {
    /// Creates a new [`JupiterApi`] client pointing at `https://api.jup.ag`, authenticated with
    /// `api_key`.
    pub fn new(api_key: String) -> Self {
        Self { base_url: "https://api.jup.ag".to_string(), api_key, client: Client::new() }
    }

    /// Fetches a Jupiter Ultra order for swapping `amount` of `sell` into `buy`, with `taker` as
    /// the signing account.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`JupiterUltraOrder`].
    pub async fn get_ultra_order(
        &self,
        sell: &str,
        buy: &str,
        amount: u64,
        taker: &str,
    ) -> Result<JupiterUltraOrder> {
        let url = format!(
            "{}/ultra/v1/order?inputMint={}&outputMint={}&amount={}&taker={}",
            &self.base_url, sell, buy, amount, taker
        );

        let order: JupiterUltraOrder = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<JupiterUltraOrder>()
            .await?;

        Ok(order)
    }

    /// Fetches current USD price data for `token` (a mint address), keyed by token id.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into `HashMap<String, TokenPrice>`.
    pub async fn get_price_quote(&self, token: &str) -> Result<HashMap<String, TokenPrice>> {
        let url = format!("{}/price/v3?ids={}", &self.base_url, token);

        let quote: HashMap<String, TokenPrice> = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<HashMap<String, TokenPrice>>()
            .await?;

        Ok(quote)
    }

    /// Requests a swap quote for exchanging `amount` of `sell` into `buy`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`JupiterSwapQuote`].
    pub async fn get_swap_quote(
        &self,
        sell: &str,
        buy: &str,
        amount: u64,
    ) -> Result<JupiterSwapQuote> {
        let url = format!(
            "{}/swap/v1/quote?inputMint={}&outputMint={}&amount={}",
            &self.base_url, sell, buy, amount
        );

        let quote: JupiterSwapQuote = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<JupiterSwapQuote>()
            .await?;

        Ok(quote)
    }

    /// Builds a swap transaction for `pubkey` from `swap_quote`, returning the serialized
    /// transaction to sign.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`SwapOrder`].
    pub async fn get_swap_order(
        &self,
        pubkey: &str,
        swap_quote: JupiterSwapQuote,
    ) -> Result<SwapOrder> {
        let url = format!("{}/swap/v1/swap", &self.base_url);

        let swap_data =
            SwapData { user_public_key: pubkey.to_string(), quote_response: swap_quote };

        let order: SwapOrder = self
            .client
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

    /// Creates a limit order from `body`, returning the serialized transaction to sign.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`JupTxn`].
    pub async fn post_limit_order(&self, body: JupLimitOrder) -> Result<JupTxn> {
        let url = format!("{}/trigger/v1/createOrder", &self.base_url);

        let resp = self
            .client
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

    /// Fetches all tokens available for lending in Jupiter Lend.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into `Vec<JupiterLendMarket>`.
    pub async fn get_lendable_tokens(&self) -> Result<Vec<JupiterLendMarket>> {
        let url = format!("{}/lend/v1/earn/tokens", self.base_url);

        let tokens = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .send()
            .await?
            .json::<Vec<JupiterLendMarket>>()
            .await?;

        Ok(tokens)
    }

    /// Deposits `body.amount` of `body.asset` into Jupiter Lend, returning the serialized
    /// transaction to sign.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`JupTxn`].
    pub async fn post_deposit(&self, body: JupEarnInput) -> Result<JupTxn> {
        let url = format!("{}/lend/v1/earn/deposit", self.base_url);

        let resp = self
            .client
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

    /// Withdraws `body.amount` of `body.asset` from Jupiter Lend, returning the serialized
    /// transaction to sign.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`JupTxn`].
    pub async fn post_withdraw(&self, body: JupEarnInput) -> Result<JupTxn> {
        let url = format!("{}/lend/v1/earn/withdraw", self.base_url);

        let resp = self
            .client
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

/// Response containing a serialized transaction to sign and submit.
#[derive(Debug, Deserialize)]
pub struct JupTxn {
    /// Base64-encoded serialized Solana transaction.
    pub transaction: String,
}

/// Current USD price data for a token.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPrice {
    /// Current price in USD.
    pub usd_price: f64,
    /// Solana block ID at which the price was observed.
    pub block_id: u64,
    /// Number of decimals of the token.
    pub decimals: u8,
    /// Price change over the last 24 hours, as a percentage.
    pub price_change_24h: f64,
}

/// Jupiter Ultra order, including route, fees, and serialized transaction.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterUltraOrder {
    /// Input amount, in the input token's smallest unit.
    pub in_amount: String,
    /// Expected output amount, in the output token's smallest unit.
    pub out_amount: String,
    /// Minimum acceptable output amount, in the output token's smallest unit.
    pub other_amount_threshold: String,
    /// Swap mode (e.g. `"ExactIn"`, `"ExactOut"`).
    pub swap_mode: String,
    /// Slippage tolerance in basis points.
    pub slippage_bps: u32,
    /// Price impact as a stringified percentage.
    pub price_impact_pct: String,
    /// Route plan hops.
    pub route_plan: Vec<RoutePlan>,
    /// Mint address of the token used for fees.
    pub fee_mint: String,
    /// Fee in basis points.
    pub fee_bps: u32,
    /// Platform fee details, if any.
    pub platform_fee: Option<PlatformFee>,
    /// Signature fee in lamports.
    pub signature_fee_lamports: u64,
    /// Account paying the signature fee, if any.
    pub signature_fee_payer: Option<String>,
    /// Prioritization fee in lamports.
    pub prioritization_fee_lamports: u64,
    /// Account paying the prioritization fee, if any.
    pub prioritization_fee_payer: Option<String>,
    /// Rent fee in lamports.
    pub rent_fee_lamports: u64,
    /// Account paying the rent fee, if any.
    pub rent_fee_payer: Option<String>,
    /// Serialized transaction to sign, if included.
    pub transaction: Option<String>,
    /// Whether the order can be executed gasless.
    pub gasless: bool,
    /// Taker account, if known.
    pub taker: Option<String>,
    /// Order mode.
    pub mode: String,
    /// Mint address of the input token.
    pub input_mint: String,
    /// Mint address of the output token.
    pub output_mint: String,
    /// Swap type.
    pub swap_type: String,
    /// Router used for the swap.
    pub router: String,
    /// Request identifier.
    pub request_id: String,
    /// USD value of the input amount.
    pub in_usd_value: f64,
    /// USD value of the output amount.
    pub out_usd_value: f64,
    /// Price impact.
    pub price_impact: f64,
    /// USD value of the swap.
    pub swap_usd_value: f64,
    /// Total time taken to build the order.
    pub total_time: u32,
}

/// Jupiter swap quote describing the route and amounts for a swap.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterSwapQuote {
    /// Mint address of the input token.
    pub input_mint: String,
    /// Input amount, in the input token's smallest unit.
    pub in_amount: String,
    /// Mint address of the output token.
    pub output_mint: String,
    /// Expected output amount, in the output token's smallest unit.
    pub out_amount: String,
    /// Minimum acceptable output amount, in the output token's smallest unit.
    pub other_amount_threshold: String,
    /// Swap mode (e.g. `"ExactIn"`, `"ExactOut"`).
    pub swap_mode: String,
    /// Slippage tolerance in basis points.
    pub slippage_bps: u32,
    /// Platform fee, if any.
    pub platform_fee: Option<String>,
    /// Price impact as a stringified percentage.
    pub price_impact_pct: String,
    /// Route plan hops.
    pub route_plan: Vec<RoutePlan>,
    /// Solana slot at which the quote was generated.
    pub context_slot: u64,
    /// Time taken to generate the quote, in seconds.
    pub time_taken: f64,
    /// USD value of the swap.
    pub swap_usd_value: String,
    /// Whether a simpler route was used.
    pub simpler_route_used: bool,
    /// Whether incurred slippage is used for quoting, if specified.
    pub use_incurred_slippage_for_quoting: Option<String>,
    /// Other candidate route plans, if included.
    pub other_route_plans: Option<String>,
    /// Whether a long-tail token was loaded.
    pub loaded_longtail_token: bool,
    /// Instruction version, if specified.
    pub instruction_version: Option<String>,
}

/// Swap transaction built by the Jupiter Swap API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapOrder {
    /// Serialized transaction to sign.
    pub swap_transaction: String,
    /// Last valid block height for the transaction.
    pub last_valid_block_height: u32,
    /// Prioritization fee in lamports.
    pub prioritization_fee_lamports: u32,
}

/// A single hop in a swap route.
#[derive(Debug, Deserialize, Serialize)]
pub struct RoutePlan {
    /// Details of the swap executed in this hop.
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    /// Percentage of the total amount routed through this hop.
    pub percent: Option<u32>,
    /// Basis points of the total amount routed through this hop.
    pub bps: Option<u32>,
}

/// Details of a single swap executed in a route hop.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    /// AMM pool account key.
    pub amm_key: String,
    /// AMM label.
    pub label: String,
    /// Mint address of the input token.
    pub input_mint: String,
    /// Mint address of the output token.
    pub output_mint: String,
    /// Input amount, in the token's smallest unit.
    pub in_amount: String,
    /// Output amount, in the token's smallest unit.
    pub out_amount: String,
}

/// Platform fee details.
#[derive(Debug, Deserialize)]
pub struct PlatformFee {
    /// Fee in basis points.
    #[serde(rename = "feeBps")]
    pub fee_bps: u32,
}

/// Request body for building a swap transaction.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapData {
    /// Public key of the user building the swap.
    pub user_public_key: String,
    /// Quote to build the transaction from.
    pub quote_response: JupiterSwapQuote,
}

/// Jupiter Lend market for a lendable token.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterLendMarket {
    /// Market identifier.
    pub id: u64,
    /// Address of the lending market account.
    pub address: String,
    /// Human-readable market name.
    pub name: String,
    /// Token symbol.
    pub symbol: String,
    /// Number of decimals of the token.
    pub decimals: u8,
    /// Address of the underlying asset.
    pub asset_address: String,
    /// Underlying asset details.
    pub asset: Asset,
    /// Total assets in the market, in the token's smallest unit.
    pub total_assets: String,
    /// Total supply in the market, in shares.
    pub total_supply: String,
    /// Shares convertible per asset.
    pub convert_to_shares: String,
    /// Assets convertible per share.
    pub convert_to_assets: String,
    /// Rewards rate.
    pub rewards_rate: String,
    /// Supply rate.
    pub supply_rate: String,
    /// Total rate (supply + rewards).
    pub total_rate: String,
    /// Difference since last rebalance.
    pub rebalance_difference: String,
    /// Liquidity supply details.
    pub liquidity_supply_data: LiquiditySupplyData,
    /// Reward token addresses.
    pub rewards: Vec<String>,
}

/// Underlying asset details for a Jupiter Lend market.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    /// Asset contract address.
    pub address: String,
    /// Chain identifier.
    pub chain_id: String,
    /// Human-readable asset name.
    pub name: String,
    /// Token symbol.
    pub symbol: String,
    /// Symbol for display in UIs.
    pub ui_symbol: String,
    /// Number of decimals.
    pub decimals: u8,
    /// URL of the token logo.
    pub logo_url: String,
    /// Current price.
    pub price: String,
    /// CoinGecko identifier.
    pub coingecko_id: String,
}

/// Liquidity supply details for a Jupiter Lend market.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquiditySupplyData {
    /// Whether interest accrual mode is enabled.
    pub mode_with_interest: bool,
    /// Current supply, in the token's smallest unit.
    pub supply: String,
    /// Current withdrawal limit.
    pub withdrawal_limit: String,
    /// Timestamp of the last update.
    pub last_update_timestamp: String,
    /// Percentage the withdrawal limit expands by.
    pub expand_percent: u64,
    /// Duration of withdrawal-limit expansion.
    pub expand_duration: String,
    /// Base withdrawal limit.
    pub base_withdrawal_limit: String,
    /// Amount withdrawable until the limit.
    pub withdrawable_until_limit: String,
    /// Currently withdrawable amount.
    pub withdrawable: String,
}

/// Input for Jupiter Lend deposit and withdraw operations.
#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct JupEarnInput {
    /// Asset to deposit or withdraw.
    pub asset: String,
    /// Public key of the signing account.
    pub signer: String,
    /// Amount, in the token's smallest unit.
    #[serde_as(as = "DisplayFromStr")]
    pub amount: u64,
}

/// Jupiter Trigger limit order request body.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JupLimitOrder {
    /// Public key of the order maker.
    pub maker: String,
    /// Public key paying for the order.
    pub payer: String,
    /// Mint address of the input token.
    pub input_mint: String,
    /// Mint address of the output token.
    pub output_mint: String,
    /// Order parameters.
    pub params: JupLimitParams,
}

/// Parameters for a Jupiter Trigger limit order.
#[serde_as]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JupLimitParams {
    /// Amount the maker offers, in the input token's smallest unit.
    #[serde_as(as = "DisplayFromStr")]
    pub making_amount: u64,
    /// Amount the maker wants in return, in the output token's smallest unit.
    #[serde_as(as = "DisplayFromStr")]
    pub taking_amount: u64,
    /// Unix timestamp (seconds) at which the order expires.
    #[serde_as(as = "DisplayFromStr")]
    pub expired_at: u64,
}
