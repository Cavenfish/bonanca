use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

pub struct DexScreenerApi {
    pub base_url: String,
}

impl DexScreenerApi {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.dexscreener.com".to_string(),
        }
    }

    pub async fn get_pair_data(&self, chain: &str, pair: &str) -> Result<DexScreenerResponse> {
        let client = Client::new();
        let url = format!("{}/latest/dex/pairs/{}/{}", &self.base_url, chain, pair);

        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<DexScreenerResponse>()
            .await?;

        Ok(resp)
    }

    pub async fn get_pairs_from_query(&self, query: &str) -> Result<DexScreenerResponse> {
        let client = Client::new();
        let url = format!("{}/latest/dex/search?q={}", &self.base_url, query);

        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<DexScreenerResponse>()
            .await?;

        Ok(resp)
    }

    pub async fn get_token_pairs(
        &self,
        chain: &str,
        token: &str,
    ) -> Result<Vec<DexScreenerPairData>> {
        let client = Client::new();
        let url = format!("{}/token-pairs/v1/{}/{}", &self.base_url, chain, token);

        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<Vec<DexScreenerPairData>>()
            .await?;

        Ok(resp)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DexScreenerResponse {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub pair: Option<DexScreenerPairData>,
    pub pairs: Option<Vec<DexScreenerPairData>>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DexScreenerPairData {
    pub chain_id: String,
    pub dex_id: String,
    pub url: String,
    pub pair_address: String,
    pub labels: Option<Vec<String>>,
    pub base_token: Token,
    pub quote_token: Token,
    #[serde_as(as = "DisplayFromStr")]
    pub price_native: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub price_usd: f64,
    pub txns: Txns,
    pub volume: Volume,
    pub price_change: PriceChange,
    pub liquidity: Option<Liquidity>,
    pub fdv: f64,
    pub market_cap: f64,
    pub pair_created_at: Option<u64>,
    pub info: Option<Info>,
    pub boosts: Option<Boosts>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Token {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Txns {
    pub m5: Option<TxnData>,
    pub h1: Option<TxnData>,
    pub h6: Option<TxnData>,
    pub h24: Option<TxnData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TxnData {
    pub buys: i32,
    pub sells: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Volume {
    pub m5: Option<f64>,
    pub h1: Option<f64>,
    pub h6: Option<f64>,
    pub h24: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceChange {
    pub m5: Option<f64>,
    pub h1: Option<f64>,
    pub h6: Option<f64>,
    pub h24: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Liquidity {
    pub usd: f64,
    pub base: f64,
    pub quote: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub image_url: Option<String>,
    pub header: Option<String>,
    pub open_graph: Option<String>,
    pub websites: Vec<Website>,
    pub socials: Vec<Social>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Boosts {
    pub active: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Website {
    pub url: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Social {
    pub url: String,
    #[serde(rename = "type")]
    pub platform: String,
}
