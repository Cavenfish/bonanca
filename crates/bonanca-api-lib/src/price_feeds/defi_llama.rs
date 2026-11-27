//! `DefiLlama` Coins API client and price quote types.

use std::collections::HashMap;

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

/// Client for the `DefiLlama` Coins REST API.
pub struct DefiLlamaApi {
    base_url: String,
    client: Client,
}

impl DefiLlamaApi {
    /// Creates a new [`DefiLlamaApi`] client pointing at `https://coins.llama.fi`.
    pub fn new() -> Self {
        Self { base_url: "https://coins.llama.fi".to_string(), client: Client::new() }
    }

    /// Fetches the current USD price quote for `address` on `chain`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`LlamaPrice`].
    pub async fn get_price_quote(&self, chain: &str, address: &str) -> Result<LlamaPrice> {
        let url = format!("{}/prices/current/{}:{}", &self.base_url, chain, address);

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<LlamaPrice>()
            .await?;

        Ok(resp)
    }
}

/// Response returned by `DefiLlama`'s current-prices endpoint.
#[derive(Debug, Deserialize)]
pub struct LlamaPrice {
    /// Map from `"{chain}:{address}"` coin key to [`CoinsData`].
    pub coins: HashMap<String, CoinsData>,
}

/// Token price and metadata entry inside [`LlamaPrice`].
#[derive(Debug, Deserialize)]
pub struct CoinsData {
    /// Token decimal places.
    pub decimals: u16,
    /// Token ticker symbol.
    pub symbol: String,
    /// Current token price in USD.
    pub price: f64,
    /// Unix timestamp of the price observation.
    pub timestamp: u64,
    /// `DefiLlama` confidence score (`0.0` to `1.0`).
    pub confidence: f32,
}
