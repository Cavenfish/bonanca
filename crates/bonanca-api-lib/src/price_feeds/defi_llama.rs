use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

pub struct DefiLlamaApi {
    pub base_url: String,
}

impl DefiLlamaApi {
    pub fn new() -> Self {
        Self {
            base_url: "https://coins.llama.fi".to_string(),
        }
    }

    pub async fn get_price_quote(&self, chain: &str, address: &str) -> Result<LlamaPrice> {
        let client = Client::new();
        let url = format!("{}/prices/current/{}:{}", &self.base_url, chain, address);

        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<LlamaPrice>()
            .await?;

        Ok(resp)
    }
}

#[derive(Debug, Deserialize)]
pub struct LlamaPrice {
    pub coins: HashMap<String, CoinsData>,
}

#[derive(Debug, Deserialize)]
pub struct CoinsData {
    pub decimals: u16,
    pub symbol: String,
    pub price: f64,
    pub timestamp: u64,
    pub confidence: f32,
}
