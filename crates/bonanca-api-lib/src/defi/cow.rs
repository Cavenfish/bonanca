use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

pub struct CowApi {
    base_url: String,
}

impl CowApi {
    pub fn new(chain: &str) -> Self {
        Self {
            base_url: format!("https://api.cow.fi/{}", chain),
        }
    }

    pub async fn get_swap_quote(&self, swap_data: &CowSwapData) -> Result<CowSwapOrder> {
        let client = Client::new();

        let url = format!("{}/api/v1/quote", self.base_url);

        let resp: CowSwapOrder = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&swap_data)
            .send()
            .await?
            .json::<CowSwapOrder>()
            .await?;

        Ok(resp)
    }

    pub async fn post_swap_order(&self, quote: &CowQuote) -> Result<String> {
        let client = Client::new();

        let url = format!("{}/api/v1/orders", self.base_url);

        let resp: String = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&quote)
            .send()
            .await?
            .text()
            .await?;
        // .json::<String>()
        // .await?;

        Ok(resp)
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CowSwapData {
    pub sell_token: String,
    pub buy_token: String,
    #[serde_as(as = "DisplayFromStr")]
    pub sell_amount_before_fee: u64,
    pub kind: String,
    pub from: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CowSwapOrder {
    pub quote: CowQuote,
    pub from: String,
    pub expiration: String,
    pub id: u64,
    pub verified: bool,
    pub protocol_fee_bps: String,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CowQuote {
    pub sell_token: String,
    pub buy_token: String,
    pub receiver: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    pub sell_amount: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub buy_amount: u64,
    pub valid_to: u64,
    pub app_data: String,
    #[serde_as(as = "DisplayFromStr")]
    pub fee_amount: u64,
    pub kind: String,
    pub partially_fillable: bool,
    pub sell_token_balance: String,
    pub buy_token_balance: String,
    pub signing_scheme: String,
    pub signature: Option<String>,
}

impl CowQuote {
    pub fn sign(&self, sig: String) -> Self {
        let mut new = self.clone();
        new.signature = Some(sig);

        new
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CowOrderPlaced {
    #[serde(rename = "UID")]
    pub uid: String,
}
