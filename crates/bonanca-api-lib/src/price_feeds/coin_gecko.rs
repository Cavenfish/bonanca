//! `CoinGecko` API v3 client and market data types.

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

/// Client for the `CoinGecko` v3 REST API.
pub struct CoinGeckoApi {
    base_url: String,
    api_key: String,
    client: Client,
}

impl CoinGeckoApi {
    /// Creates a new [`CoinGeckoApi`] client authenticated with a demo `api_key`.
    pub fn new(api_key: String) -> Self {
        Self {
            base_url: "https://api.coingecko.com/api/v3".to_string(),
            api_key,
            client: Client::new(),
        }
    }

    /// Searches `CoinGecko` for coins and NFTs matching `symbol`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`TokenSearchData`].
    pub async fn get_token_info(&self, symbol: &str) -> Result<TokenSearchData> {
        let url = format!("{}/search?query={}", &self.base_url, symbol);

        let resp = self
            .client
            .get(&url)
            .header("x-cg-demo-api-key", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<TokenSearchData>()
            .await?;

        Ok(resp)
    }

    /// Fetches the current price and market cap for `id` in `currency` with decimal `precision`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body is not valid JSON.
    ///
    /// # Panics
    ///
    /// Panics if the JSON response does not contain numeric `resp[id][currency]` or
    /// `resp[id]["{currency}_market_cap"]` fields (for example, on an unknown coin `id`,
    /// unsupported `currency`, or API error payload).
    pub async fn get_token_price(
        &self,
        currency: &str,
        id: &str,
        precision: &str,
    ) -> Result<CoinPriceData> {
        let url = format!(
            "{}/simple/price?vs_currencies={currency}&ids={id}&precision={precision}&include_market_cap=true",
            &self.base_url
        );

        let resp: Value = self
            .client
            .get(&url)
            .header("x-cg-demo-api-key", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?
            .json()
            .await?;

        let mc_string = format!("{currency}_market_cap");
        let price = resp[id][currency].as_f64().unwrap();
        let market_cap = resp[id][mc_string].as_f64().unwrap();

        Ok(CoinPriceData { price, market_cap })
    }

    /// Fetches OHLC candlestick data for coin `id` denominated in `currency` over `days` with
    /// decimal `precision`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// as `Vec<Vec<Value>>`.
    ///
    /// # Panics
    ///
    /// Panics if any returned candle array has fewer than 5 elements or if the OHLC entries at
    /// indices `1..=4` are not JSON numbers representable as `f64`.
    pub async fn get_ohlc_by_id(
        &self,
        currency: &str,
        id: &str,
        days: &str,
        precision: &str,
    ) -> Result<Vec<TokenOhlcData>> {
        let url = format!(
            "{}/coins/{id}/ohlc?vs_currency={currency}&days={days}&precision={precision}",
            &self.base_url
        );

        let resp = self
            .client
            .get(&url)
            .header("x-cg-demo-api-key", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<Vec<Vec<Value>>>()
            .await?;

        let data: Vec<TokenOhlcData> = resp
            .iter()
            .map(|v| TokenOhlcData {
                open: v.get(1).unwrap().as_f64().unwrap(),
                high: v.get(2).unwrap().as_f64().unwrap(),
                low: v.get(3).unwrap().as_f64().unwrap(),
                close: v.get(4).unwrap().as_f64().unwrap(),
            })
            .collect();

        Ok(data)
    }
}

/// Single OHLC candlestick returned by [`CoinGeckoApi::get_ohlc_by_id`].
#[derive(Debug, Clone)]
pub struct TokenOhlcData {
    /// Opening price for the candle interval.
    pub open: f64,
    /// Highest price reached during the candle interval.
    pub high: f64,
    /// Lowest price reached during the candle interval.
    pub low: f64,
    /// Closing price for the candle interval.
    pub close: f64,
}

/// Search results returned by [`CoinGeckoApi::get_token_info`].
#[derive(Debug, Clone, Deserialize)]
pub struct TokenSearchData {
    /// Matching coins returned by the search query.
    pub coins: Vec<Coin>,
    /// Matching NFT collections returned by the search query.
    pub nfts: Vec<Nft>,
}

/// Coin entry returned inside [`TokenSearchData`].
#[derive(Debug, Clone, Deserialize)]
pub struct Coin {
    /// `CoinGecko` coin ID.
    pub id: String,
    /// Human-readable coin name.
    pub name: String,
    /// API-specific symbol identifier.
    pub api_symbol: String,
    /// Coin ticker symbol.
    pub symbol: String,
    /// Optional rank by market capitalization.
    pub market_cap_rank: Option<u32>,
    /// URL of the coin's small thumbnail icon.
    pub thumb: String,
    /// URL of the coin's large icon.
    pub large: String,
}

/// NFT collection entry returned inside [`TokenSearchData`].
#[derive(Debug, Clone, Deserialize)]
pub struct Nft {
    /// `CoinGecko` NFT collection ID.
    pub id: String,
    /// Collection name.
    pub name: String,
    /// Collection ticker symbol.
    pub symbol: String,
    /// URL of the collection's thumbnail image.
    pub thumb: String,
}

/// Current price and market capitalization returned by [`CoinGeckoApi::get_token_price`].
#[derive(Debug, Clone, Deserialize)]
pub struct CoinPriceData {
    /// Spot price in the requested quote currency.
    pub price: f64,
    /// Total market capitalization in the requested quote currency.
    pub market_cap: f64,
}
