//! `DexScreener` API client and trading pair response types.

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

/// Client for the `DexScreener` REST API.
pub struct DexScreenerApi {
    base_url: String,
    client: Client,
}

impl DexScreenerApi {
    /// Creates a new [`DexScreenerApi`] client pointing at `https://api.dexscreener.com`.
    pub fn new() -> Self {
        Self { base_url: "https://api.dexscreener.com".to_string(), client: Client::new() }
    }

    /// Fetches market data for a specific liquidity `pair` address on `chain`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`DexScreenerResponse`].
    pub async fn get_pair_data(&self, chain: &str, pair: &str) -> Result<DexScreenerResponse> {
        let url = format!("{}/latest/dex/pairs/{}/{}", &self.base_url, chain, pair);

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<DexScreenerResponse>()
            .await?;

        Ok(resp)
    }

    /// Searches for DEX pairs matching `query` (symbol, name, or address).
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`DexScreenerResponse`].
    pub async fn get_pairs_from_query(&self, query: &str) -> Result<DexScreenerResponse> {
        let url = format!("{}/latest/dex/search?q={}", &self.base_url, query);

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<DexScreenerResponse>()
            .await?;

        Ok(resp)
    }

    /// Fetches all liquidity pairs for `token` on `chain`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into `Vec<DexScreenerPairData>`.
    pub async fn get_token_pairs(
        &self,
        chain: &str,
        token: &str,
    ) -> Result<Vec<DexScreenerPairData>> {
        let url = format!("{}/token-pairs/v1/{}/{}", &self.base_url, chain, token);

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<Vec<DexScreenerPairData>>()
            .await?;

        Ok(resp)
    }
}

/// Response envelope returned by `DexScreener` pair lookup and search endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct DexScreenerResponse {
    /// API schema version string.
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    /// Single matching pair returned by pair-by-address lookups.
    pub pair: Option<DexScreenerPairData>,
    /// List of matching pairs returned by search or multi-pair lookups.
    pub pairs: Option<Vec<DexScreenerPairData>>,
}

/// Liquidity pair market data returned by `DexScreener`.
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DexScreenerPairData {
    /// Chain identifier (for example, `"base"` or `"solana"`).
    pub chain_id: String,
    /// DEX identifier (for example, `"uniswap"` or `"raydium"`).
    pub dex_id: String,
    /// Canonical `DexScreener` web URL for the pair.
    pub url: String,
    /// On-chain address of the liquidity pool or pair.
    pub pair_address: String,
    /// Optional pool labels (such as fee tier or version tags).
    pub labels: Option<Vec<String>>,
    /// Base token metadata.
    pub base_token: Token,
    /// Quote token metadata.
    pub quote_token: Token,
    /// Price of the base token denominated in the quote token.
    #[serde_as(as = "DisplayFromStr")]
    pub price_native: f64,
    /// Price of the base token in USD.
    #[serde_as(as = "DisplayFromStr")]
    pub price_usd: f64,
    /// Buy/sell transaction counts over recent time windows.
    pub txns: Txns,
    /// Trading volume in USD over recent time windows.
    pub volume: Volume,
    /// Percentage price changes over recent time windows.
    pub price_change: PriceChange,
    /// Pool liquidity metrics, if reported.
    pub liquidity: Option<Liquidity>,
    /// Fully diluted valuation in USD.
    pub fdv: f64,
    /// Market capitalization in USD.
    pub market_cap: f64,
    /// Unix timestamp in milliseconds when the pair was created, if available.
    pub pair_created_at: Option<u64>,
    /// Optional token project links and media metadata.
    pub info: Option<Info>,
    /// Active `DexScreener` boost status, if present.
    pub boosts: Option<Boosts>,
}

/// Token metadata inside [`DexScreenerPairData`].
#[derive(Debug, Clone, Deserialize)]
pub struct Token {
    /// On-chain token address.
    pub address: String,
    /// Human-readable token name.
    pub name: String,
    /// Token ticker symbol.
    pub symbol: String,
}

/// Buy and sell transaction counts across rolling time intervals.
#[derive(Debug, Clone, Deserialize)]
pub struct Txns {
    /// Last 5 minutes.
    pub m5: Option<TxnData>,
    /// Last 1 hour.
    pub h1: Option<TxnData>,
    /// Last 6 hours.
    pub h6: Option<TxnData>,
    /// Last 24 hours.
    pub h24: Option<TxnData>,
}

/// Buy and sell counts for a single time window.
#[derive(Debug, Clone, Deserialize)]
pub struct TxnData {
    /// Number of buy transactions.
    pub buys: i32,
    /// Number of sell transactions.
    pub sells: i32,
}

/// USD trading volume across rolling time intervals.
#[derive(Debug, Clone, Deserialize)]
pub struct Volume {
    /// Last 5 minutes.
    pub m5: Option<f64>,
    /// Last 1 hour.
    pub h1: Option<f64>,
    /// Last 6 hours.
    pub h6: Option<f64>,
    /// Last 24 hours.
    pub h24: Option<f64>,
}

/// Percentage price change across rolling time intervals.
#[derive(Debug, Clone, Deserialize)]
pub struct PriceChange {
    /// Last 5 minutes.
    pub m5: Option<f64>,
    /// Last 1 hour.
    pub h1: Option<f64>,
    /// Last 6 hours.
    pub h6: Option<f64>,
    /// Last 24 hours.
    pub h24: Option<f64>,
}

/// Liquidity depth for a DEX pair.
#[derive(Debug, Clone, Deserialize)]
pub struct Liquidity {
    /// Total pool liquidity in USD.
    pub usd: f64,
    /// Base token reserve in the pool.
    pub base: f64,
    /// Quote token reserve in the pool.
    pub quote: f64,
}

/// Project branding and social metadata inside [`DexScreenerPairData`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    /// Token icon URL.
    pub image_url: Option<String>,
    /// Banner header image URL.
    pub header: Option<String>,
    /// `OpenGraph` preview image URL.
    pub open_graph: Option<String>,
    /// Project website links.
    pub websites: Vec<Website>,
    /// Social media links.
    pub socials: Vec<Social>,
}

/// Boost metadata for a promoted `DexScreener` pair.
#[derive(Debug, Clone, Deserialize)]
pub struct Boosts {
    /// Number of active boosts, if any.
    pub active: Option<i32>,
}

/// Website entry inside [`Info`].
#[derive(Debug, Clone, Deserialize)]
pub struct Website {
    /// Website URL.
    pub url: String,
    /// Display label for the website.
    pub label: String,
}

/// Social media entry inside [`Info`].
#[derive(Debug, Clone, Deserialize)]
pub struct Social {
    /// Profile or community URL.
    pub url: String,
    /// Platform name (mapped from JSON `"type"`).
    #[serde(rename = "type")]
    pub platform: String,
}
