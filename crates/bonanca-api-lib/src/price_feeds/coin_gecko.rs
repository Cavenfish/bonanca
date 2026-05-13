use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

pub struct CoinGeckoApi {
    base_url: String,
    api_key: String,
    client: Client,
}

impl CoinGeckoApi {
    pub fn new(api_key: String) -> Self {
        Self {
            base_url: "https://api.coingecko.com/api/v3".to_string(),
            api_key,
            client: Client::new(),
        }
    }

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

#[derive(Debug, Clone)]
pub struct TokenOhlcData {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenSearchData {
    pub coins: Vec<Coin>,
    pub nfts: Vec<Nft>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Coin {
    pub id: String,
    pub name: String,
    pub api_symbol: String,
    pub symbol: String,
    pub market_cap_rank: Option<u32>,
    pub thumb: String,
    pub large: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Nft {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub thumb: String,
}
