use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

pub struct CoinMarketCap {
    pub base_url: String,
    pub api_key: String,
}

impl CoinMarketCap {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url: base_url,
            api_key: api_key,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CmcResponse {
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
    pub amount: i32,
    pub last_updated: Option<String>,
    pub quote: Quote,
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    pub USD: UsdQuote,
}

#[derive(Debug, Deserialize)]
pub struct UsdQuote {
    pub price: Option<f64>,
    pub last_updated: Option<String>,
}
