//! `CoinMarketCap` Pro API client and price conversion types.

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

/// Client for the `CoinMarketCap` Pro REST API.
pub struct CoinMarketCapApi {
    base_url: String,
    api_key: String,
    client: Client,
}

impl CoinMarketCapApi {
    /// Creates a new [`CoinMarketCapApi`] client authenticated with `api_key`.
    pub fn new(api_key: String) -> Self {
        Self {
            base_url: "https://pro-api.coinmarketcap.com".to_string(),
            api_key,
            client: Client::new(),
        }
    }

    /// Converts `amount` of `token` (by ticker symbol) into a USD price quote.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`CmcPriceQuote`].
    pub async fn get_price_quote(&self, token: &str, amount: f64) -> Result<CmcPriceQuote> {
        let url = format!(
            "{}/v2/tools/price-conversion?symbol={}&amount={}&convert=USD",
            &self.base_url, token, amount
        );

        let resp = self
            .client
            .get(&url)
            .header("X-CMC_PRO_API_KEY", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<CmcPriceQuote>()
            .await?;

        Ok(resp)
    }
}

/// Response returned by `CoinMarketCap`'s v2 price-conversion endpoint.
#[derive(Debug, Deserialize)]
pub struct CmcPriceQuote {
    /// Request status metadata.
    pub status: Status,
    /// Converted quote entries matching the requested symbol.
    pub data: Vec<TokenData>,
}

/// API status metadata included in `CoinMarketCap` responses.
#[derive(Debug, Deserialize)]
pub struct Status {
    /// ISO-8601 timestamp when the response was generated.
    pub timestamp: String,
    /// Numeric error code (`0` on success).
    pub error_code: i32,
    /// Optional error description when `error_code` is non-zero.
    pub error_message: Option<String>,
    /// Server processing duration in milliseconds.
    pub elapsed: i32,
    /// API credits consumed by this request.
    pub credit_count: i32,
    /// Optional informational notice from the API.
    pub notice: Option<String>,
}

/// Token metadata and converted quote returned inside [`CmcPriceQuote`].
#[derive(Debug, Deserialize)]
pub struct TokenData {
    /// `CoinMarketCap` numeric asset ID.
    pub id: i64,
    /// Token ticker symbol.
    pub symbol: String,
    /// Human-readable token name.
    pub name: String,
    /// Input token quantity that was converted.
    pub amount: f64,
    /// ISO-8601 timestamp of the last price update.
    pub last_updated: Option<String>,
    /// Converted quote values keyed by target currency.
    pub quote: Quote,
}

/// Currency quote container inside [`TokenData`].
#[derive(Debug, Deserialize)]
pub struct Quote {
    /// Quote denominated in US dollars.
    #[serde(rename = "USD")]
    pub usd: UsdQuote,
}

/// USD-denominated conversion quote inside [`Quote`].
#[derive(Debug, Deserialize)]
pub struct UsdQuote {
    /// Converted USD value for the requested token amount, if available.
    pub price: Option<f64>,
    /// ISO-8601 timestamp of the USD quote update.
    pub last_updated: Option<String>,
}
