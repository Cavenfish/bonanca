use std::str;

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QueryData {
    pub data: TokenData,
}

#[derive(Debug, Deserialize)]
pub struct TokenData {
    pub id: u64,
    pub symbol: String,
    pub name: String,

    pub quote: std::collections::HashMap<String, Quote>,
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    pub price: f64,
}

pub async fn get_token_value(id: u64, amount: f64, api_url: &str, api_key: &str) -> Result<f64> {
    let url = format!(
        "{}/v2/tools/price-conversion?id={}&amount={}&convert=USD",
        api_url, id, amount
    );

    let client = Client::new();
    let resp = client
        .get(&url)
        .header("X-CMC_PRO_API_KEY", api_key)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<QueryData>()
        .await?;

    let price = resp.data.quote.get("USD").unwrap().price;

    Ok(price)
}
