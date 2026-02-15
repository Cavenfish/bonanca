use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

pub struct CowApi {
    base_url: String,
    client: Client,
}

impl CowApi {
    pub fn new(chain: &str) -> Self {
        Self {
            base_url: format!("https://api.cow.fi/{}", chain),
            client: Client::new(),
        }
    }

    pub async fn get_order_info(&self, uid: &str) -> Result<CowSwapPlacedOrder> {
        let url = format!("{}/api/v1/orders/{}", self.base_url, uid);

        let resp = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await?
            .json::<CowSwapPlacedOrder>()
            .await?;

        Ok(resp)
    }

    pub async fn get_user_orders(
        &self,
        user: &str,
        limit: Option<u16>,
    ) -> Result<Vec<CowSwapPlacedOrder>> {
        let url = format!(
            "{}/api/v1/account/{}/orders?limit={}",
            self.base_url,
            user,
            limit.unwrap_or(10)
        );

        let resp = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await?
            .json::<Vec<CowSwapPlacedOrder>>()
            .await?;

        Ok(resp)
    }

    pub async fn get_swap_quote(&self, swap_data: &CowSwapData) -> Result<CowSwapOrder> {
        let url = format!("{}/api/v1/quote", self.base_url);

        let mut resp: CowSwapOrder = self
            .client
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
        let url = format!("{}/api/v1/orders", self.base_url);

        let resp: String = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&order)
            .send()
            .await?
            .text()
            .await?;

        let clean = resp.trim_matches(|c| c == '"').to_string();

        Ok(clean)
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

impl CowQuote {
    pub fn build_quote(
        sell_token: &str,
        buy_token: &str,
        receiver: Option<String>,
        sell_amount: u64,
        buy_amount: u64,
        valid_to: u32,
    ) -> Self {
        Self {
            sell_token: sell_token.to_string(),
            buy_token: buy_token.to_string(),
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data: "{}".to_string(), // hash below is keccak256("{}")
            app_data_hash: "0xb48d38f93eaa084033fc5970bf96e559c33c4cdc07d889ab00b4d63f9590739d"
                .to_string(),
            fee_amount: 0,
            kind: "sell".to_string(),
            partially_fillable: true,
            sell_token_balance: "erc20".to_string(),
            buy_token_balance: "erc20".to_string(),
            signing_scheme: "eip712".to_string(),
        }
    }

    pub fn sign(self, sig: String, from: String) -> CowOrder {
        CowOrder {
            sell_token: self.sell_token,
            buy_token: self.buy_token,
            receiver: self.receiver,
            sell_amount: self.sell_amount,
            buy_amount: self.buy_amount,
            valid_to: self.valid_to,
            app_data: self.app_data,
            app_data_hash: self.app_data_hash,
            fee_amount: self.fee_amount,
            kind: self.kind,
            partially_fillable: self.partially_fillable,
            sell_token_balance: self.sell_token_balance,
            buy_token_balance: self.buy_token_balance,
            signing_scheme: self.signing_scheme,
            signature: sig,
            from: from,
        }
    }
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CowSwapPlacedOrder {
    pub creation_date: String,
    pub owner: String,
    pub uid: String,
    pub available_balance: Option<String>,
    pub executed_buy_amount: String,
    pub executed_sell_amount: String,
    pub executed_sell_amount_before_fees: String,
    pub executed_fee_amount: String,
    pub executed_fee: String,
    pub executed_fee_token: String,
    pub invalidated: bool,
    pub status: String,
    pub class: String,
    pub settlement_contract: String,
    pub is_liquidity_order: bool,
    pub full_app_data: String,
    pub quote: Option<CowSwapExistingQuote>,
    pub sell_token: String,
    pub buy_token: String,
    pub receiver: String,
    pub sell_amount: String,
    pub buy_amount: String,
    pub valid_to: u64,
    pub app_data: String,
    pub fee_amount: String,
    pub kind: String,
    pub partially_fillable: bool,
    pub sell_token_balance: String,
    pub buy_token_balance: String,
    pub signing_scheme: String,
    pub signature: String,
    pub interactions: Interactions,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CowSwapExistingQuote {
    pub gas_amount: String,
    pub gas_price: String,
    pub sell_token_price: String,
    pub sell_amount: String,
    pub buy_amount: String,
    pub fee_amount: String,
    pub solver: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Interactions {
    pub pre: Vec<String>,
    pub post: Vec<String>,
}
