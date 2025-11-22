use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::{api_lib::traits::Oracle, holdings::Asset};

pub struct CoinMarketCap {
    pub base_url: String,
    pub api_key: String,
}

impl CoinMarketCap {
    pub fn new(base_url: String, key: Option<String>) -> Self {
        let api_key = key.unwrap();
        Self { base_url, api_key }
    }

    pub async fn get_price_quote(&self, token: &str, amount: f64) -> Result<CmcPriceQuote> {
        let client = Client::new();
        let url = format!(
            "{}/v2/tools/price-conversion?symbol={}&amount={}&convert=USD",
            &self.base_url, token, amount
        );

        let resp = client
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

#[async_trait]
impl Oracle for CoinMarketCap {
    async fn get_token_value(&self, asset: &Asset, amount: f64) -> Result<f64> {
        let quote = self.get_price_quote(&asset.symbol, amount).await?;

        let data = quote.data.first().unwrap();

        let value = data.quote.usd.price.unwrap();

        Ok(value)
    }
}

#[derive(Debug, Deserialize)]
pub struct CmcPriceQuote {
    pub status: Status,
    pub data: Vec<TokenData>,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub timestamp: String,
    pub error_code: i32,
    pub error_message: Option<String>,
    pub elapsed: i32,
    pub credit_count: i32,
    pub notice: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TokenData {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub amount: f64,
    pub last_updated: Option<String>,
    pub quote: Quote,
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    #[serde(rename = "USD")]
    pub usd: UsdQuote,
}

#[derive(Debug, Deserialize)]
pub struct UsdQuote {
    pub price: Option<f64>,
    pub last_updated: Option<String>,
}
