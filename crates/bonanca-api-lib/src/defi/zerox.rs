//! 0x Swap API v2 client and quote types.

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

/// Client for the 0x Swap API v2.
pub struct ZeroXApi {
    base_url: String,
    api_key: String,
    chain_id: u16,
    client: Client,
}

impl ZeroXApi {
    /// Creates a new [`ZeroXApi`] client pointing at `https://api.0x.org`, authenticated with
    /// `api_key`, targeting `chain_id`.
    pub fn new(api_key: String, chain_id: u16) -> Self {
        Self {
            base_url: "https://api.0x.org".to_string(),
            api_key,
            chain_id,
            client: Client::new(),
        }
    }

    /// Fetches an indicative price quote for selling `amount` of `sell` for `buy`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`ZeroXPriceQuote`].
    pub async fn get_price_quote(
        &self,
        sell: &str,
        buy: &str,
        amount: &str,
    ) -> Result<ZeroXPriceQuote> {
        let url = format!(
            "{}/swap/allowance-holder/price?chainId={}&sellToken={}&sellAmount={}&buyToken={}",
            &self.base_url, &self.chain_id, sell, amount, buy
        );

        let quote: ZeroXPriceQuote = self
            .client
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

    /// Fetches a firm swap quote for selling `amount` of `sell` for `buy`, with `taker` as the
    /// account that would execute the swap.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`ZeroXSwapQuote`].
    pub async fn get_swap_quote(
        &self,
        sell: &str,
        buy: &str,
        amount: &str,
        taker: &str,
    ) -> Result<ZeroXSwapQuote> {
        let url = format!(
            "{}/swap/allowance-holder/quote?chainId={}&sellToken={}&sellAmount={}&buyToken={}&taker={}",
            &self.base_url, &self.chain_id, sell, amount, buy, taker,
        );

        let quote: ZeroXSwapQuote = self
            .client
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

/// Indicative price quote from the 0x Swap API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXPriceQuote {
    /// Address that needs allowance to spend `sell_token`.
    pub allowance_target: String,
    /// Block number at which the quote was generated.
    pub block_number: String,
    /// Expected amount of `buy_token`, in the token's smallest unit.
    pub buy_amount: String,
    /// Address of the token being bought.
    pub buy_token: String,
    /// Fee breakdown.
    pub fees: Fees,
    /// Detected issues with the swap.
    pub issues: Issues,
    /// Whether sufficient liquidity is available.
    pub liquidity_available: bool,
    /// Minimum acceptable amount of `buy_token`, in the token's smallest unit.
    pub min_buy_amount: String,
    /// Swap route details.
    pub route: Route,
    /// Amount of `sell_token`, in the token's smallest unit.
    pub sell_amount: String,
    /// Address of the token being sold.
    pub sell_token: String,
    /// Token metadata.
    pub token_metadata: TokenMetadata,
    /// Total network fee estimate.
    pub total_network_fee: String,
    /// Quote identifier.
    pub zid: String,
}

/// Firm swap quote from the 0x Swap API, including a ready-to-submit transaction.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXSwapQuote {
    /// Address that needs allowance to spend `sell_token`.
    pub allowance_target: String,
    /// Block number at which the quote was generated.
    pub block_number: String,
    /// Expected amount of `buy_token`, in the token's smallest unit.
    pub buy_amount: String,
    /// Address of the token being bought.
    pub buy_token: String,
    /// Fee breakdown.
    pub fees: Fees,
    /// Detected issues with the swap.
    pub issues: Issues,
    /// Whether sufficient liquidity is available.
    pub liquidity_available: bool,
    /// Minimum acceptable amount of `buy_token`, in the token's smallest unit.
    pub min_buy_amount: String,
    /// Swap route details.
    pub route: Route,
    /// Amount of `sell_token`, in the token's smallest unit.
    pub sell_amount: String,
    /// Address of the token being sold.
    pub sell_token: String,
    /// Token metadata.
    pub token_metadata: TokenMetadata,
    /// Total network fee estimate.
    pub total_network_fee: String,
    /// Transaction to submit.
    pub transaction: Transaction,
    /// Quote identifier.
    pub zid: String,
}

/// Fee breakdown for a swap.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    /// Fee charged by the integrator, if any.
    pub integrator_fee: Option<String>,
    /// Fee charged by 0x, if any.
    pub zero_ex_fee: Option<ZeroExFee>,
    /// Gas fee, if any.
    pub gas_fee: Option<String>,
}

/// Fee charged by 0x.
#[derive(Debug, Deserialize)]
pub struct ZeroExFee {
    /// Fee amount.
    pub amount: String,
    /// Address of the token the fee is paid in.
    pub token: String,
    /// Fee type.
    #[serde(rename = "type")]
    pub fee_type: String,
}

/// Issues detected with a swap.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issues {
    /// Allowance issue, if any.
    pub allowance: Option<Allowance>,
    /// Balance issue, if any.
    pub balance: Option<Balance>,
    /// Whether swap simulation was incomplete.
    pub simulation_incomplete: bool,
    /// Sources passed by the caller that are not valid.
    pub invalid_sources_passed: Vec<String>,
}

/// Allowance issue details.
#[derive(Debug, Deserialize)]
pub struct Allowance {
    /// Current allowance, in the token's smallest unit.
    pub actual: String,
    /// Address of the spender.
    pub spender: String,
}

/// Balance issue details.
#[derive(Debug, Deserialize)]
pub struct Balance {
    /// Address of the token.
    pub token: String,
    /// Current balance, in the token's smallest unit.
    pub actual: String,
    /// Required balance, in the token's smallest unit.
    pub expected: String,
}

/// Swap route details.
#[derive(Debug, Deserialize)]
pub struct Route {
    /// Individual fills composing the route.
    pub fills: Vec<Fill>,
    /// Tokens involved in the route.
    pub tokens: Vec<Token>,
}

/// A single fill in a swap route.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
    /// Address of the token being sold in this fill.
    pub from: String,
    /// Address of the token being bought in this fill.
    pub to: String,
    /// Liquidity source name.
    pub source: String,
    /// Proportion routed through this fill, in basis points.
    pub proportion_bps: String,
}

/// Token involved in a swap route.
#[derive(Debug, Deserialize)]
pub struct Token {
    /// Token contract address.
    pub address: String,
    /// Token symbol.
    pub symbol: String,
}

/// Token metadata for the swap pair.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    /// Metadata for the bought token.
    pub buy_token: TokenTax,
    /// Metadata for the sold token.
    pub sell_token: TokenTax,
}

/// Token tax information.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTax {
    /// Buy tax in basis points.
    pub buy_tax_bps: String,
    /// Sell tax in basis points.
    pub sell_tax_bps: String,
}

/// Transaction to submit for the swap.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// Destination address.
    pub to: String,
    /// Calldata.
    pub data: String,
    /// Gas limit.
    pub gas: String,
    /// Gas price.
    pub gas_price: String,
    /// Native value to send.
    pub value: String,
}
