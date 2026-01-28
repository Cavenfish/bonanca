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

        let mut resp: CowSwapOrder = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&swap_data)
            .send()
            .await?
            .json::<CowSwapOrder>()
            .await?;

        // Fee has to be zero even though they give it non-zero
        resp.quote.fee_amount = 0;

        Ok(resp)
    }

    pub async fn post_swap_order(&self, order: &CowOrder) -> Result<String> {
        let client = Client::new();

        let url = format!("{}/api/v1/orders", self.base_url);

        let resp: String = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&order)
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
    pub receiver: String,
    pub app_data: String,
    pub app_data_hash: String,
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

impl CowSwapOrder {
    pub fn sign(self, sig: String) -> CowOrder {
        CowOrder {
            sell_token: self.quote.sell_token,
            buy_token: self.quote.buy_token,
            receiver: self.quote.receiver,
            sell_amount: self.quote.sell_amount,
            buy_amount: self.quote.buy_amount,
            valid_to: self.quote.valid_to,
            app_data: self.quote.app_data,
            app_data_hash: self.quote.app_data_hash,
            fee_amount: self.quote.fee_amount,
            kind: self.quote.kind,
            partially_fillable: self.quote.partially_fillable,
            sell_token_balance: self.quote.sell_token_balance,
            buy_token_balance: self.quote.buy_token_balance,
            signing_scheme: self.quote.signing_scheme,
            signature: sig,
            from: self.from,
        }
    }
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
    pub valid_to: u32,
    pub app_data: String,
    pub app_data_hash: String,
    #[serde_as(as = "DisplayFromStr")]
    pub fee_amount: u64,
    pub kind: String,
    pub partially_fillable: bool,
    pub sell_token_balance: String,
    pub buy_token_balance: String,
    pub signing_scheme: String,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CowOrder {
    pub sell_token: String,
    pub buy_token: String,
    pub receiver: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    pub sell_amount: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub buy_amount: u64,
    pub valid_to: u32,
    pub app_data: String,
    pub app_data_hash: String,
    #[serde_as(as = "DisplayFromStr")]
    pub fee_amount: u64,
    pub kind: String,
    pub partially_fillable: bool,
    pub sell_token_balance: String,
    pub buy_token_balance: String,
    pub signing_scheme: String,
    pub signature: String,
    pub from: String,
}
