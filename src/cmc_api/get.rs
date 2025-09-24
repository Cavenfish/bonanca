use std::str;

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QueryData {
    pub data: std::collections::HashMap<String, TokenData>,
}

#[derive(Debug, Deserialize)]
pub struct TokenData {
    pub symbol: String,
    pub id: String,
    pub name: String,

    #[serde(flatten)]
    pub quote: std::collections::HashMap<String, Quote>,
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    pub price: f64,
    pub market_cap: f64,
}

pub async fn get_token_price(symbol: &str, api_url: &str, api_key: &str) -> Result<QueryData> {
    let url = format!(
        "{}/v2/cryptocurrency/quotes/latest?symbol={}",
        api_url, symbol
    );

    println!("{}", url);

    let client = Client::new();
    let resp = client
        .get(&url)
        .header("X-CMC_PRO_API_KEY", api_key)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<QueryData>()
        .await?;

    println!("{:?}", resp);

    Ok(resp)
}
