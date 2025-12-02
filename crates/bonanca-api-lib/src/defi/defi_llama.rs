use anyhow::Result;
use async_trait::async_trait;
use bonanca_core::{holdings::Asset, traits::Oracle};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

pub struct DefiLlama {
    pub base_url: String,
}

impl DefiLlama {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
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

#[async_trait]
impl Oracle for DefiLlama {
    async fn get_token_value(&self, asset: &Asset, amount: f64, chain: &str) -> Result<f64> {
        let quote = self.get_price_quote(chain, &asset.address).await?;
        let price = quote.coins.values().next().unwrap().price;
        let value = price * amount;

        Ok(value)
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
