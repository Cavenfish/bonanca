use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

pub struct ZeroXApi {
    pub base_url: String,
    pub api_key: String,
    pub chain_id: u16,
}

impl ZeroXApi {
    pub fn new(api_key: String, chain_id: u16) -> Self {
        Self {
            base_url: "https://api.0x.org".to_string(),
            api_key,
            chain_id,
        }
    }

    pub async fn get_price_quote(
        &self,
        sell: &str,
        buy: &str,
        amount: u64,
    ) -> Result<ZeroXPriceQuote> {
        let client = Client::new();

        let url = format!(
            "{}/swap/allowance-holder/price?chainId={}&sellToken={}&sellAmount={}&buyToken={}",
            &self.base_url, &self.chain_id, sell, amount, buy
        );

        let quote: ZeroXPriceQuote = client
            .get(&url)
            .header("0x-api-key", &self.api_key)
            .header("0x-version", "v2")
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<ZeroXPriceQuote>()
            .await?;

        Ok(quote)
    }

    pub async fn get_swap_quote(
        &self,
        sell: &str,
        buy: &str,
        amount: u64,
        taker: &str,
    ) -> Result<ZeroXSwapQuote> {
        let client = Client::new();

        let url = format!(
            "{}/swap/allowance-holder/quote?chainId={}&sellToken={}&sellAmount={}&buyToken={}&taker={}",
            &self.base_url, &self.chain_id, sell, amount, buy, taker,
        );

        let quote: ZeroXSwapQuote = client
            .get(&url)
            .header("0x-api-key", &self.api_key)
            .header("0x-version", "v2")
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<ZeroXSwapQuote>()
            .await?;

        Ok(quote)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXPriceQuote {
    pub allowance_target: String,
    pub block_number: String,
    pub buy_amount: String,
    pub buy_token: String,
    pub fees: Fees,
    pub issues: Issues,
    pub liquidity_available: bool,
    pub min_buy_amount: String,
    pub route: Route,
    pub sell_amount: String,
    pub sell_token: String,
    pub token_metadata: TokenMetadata,
    pub total_network_fee: String,
    pub zid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXSwapQuote {
    pub allowance_target: String,
    pub block_number: String,
    pub buy_amount: String,
    pub buy_token: String,
    pub fees: Fees,
    pub issues: Issues,
    pub liquidity_available: bool,
    pub min_buy_amount: String,
    pub route: Route,
    pub sell_amount: String,
    pub sell_token: String,
    pub token_metadata: TokenMetadata,
    pub total_network_fee: String,
    pub transaction: Transaction,
    pub zid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    pub integrator_fee: Option<String>,
    pub zero_ex_fee: Option<ZeroExFee>,
    pub gas_fee: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ZeroExFee {
    pub amount: String,
    pub token: String,
    #[serde(rename = "type")]
    pub fee_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issues {
    pub allowance: Option<Allowance>,
    pub balance: Option<Balance>,
    pub simulation_incomplete: bool,
    pub invalid_sources_passed: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Allowance {
    pub actual: String,
    pub spender: String,
}

#[derive(Debug, Deserialize)]
pub struct Balance {
    pub token: String,
    pub actual: String,
    pub expected: String,
}

#[derive(Debug, Deserialize)]
pub struct Route {
    pub fills: Vec<Fill>,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
    pub from: String,
    pub to: String,
    pub source: String,
    pub proportion_bps: String,
}

#[derive(Debug, Deserialize)]
pub struct Token {
    pub address: String,
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    pub buy_token: TokenTax,
    pub sell_token: TokenTax,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTax {
    pub buy_tax_bps: String,
    pub sell_tax_bps: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub to: String,
    pub data: String,
    pub gas: String,
    pub gas_price: String,
    pub value: String,
}
