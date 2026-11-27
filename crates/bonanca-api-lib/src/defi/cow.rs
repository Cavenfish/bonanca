//! `CoW` Protocol Order Book API client and order types.

use alloy_primitives::U256;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

/// Client for the `CoW` Protocol Order Book API.
pub struct CowApi {
    base_url: String,
    client: Client,
}

impl CowApi {
    /// Creates a new [`CowApi`] client for `chain` (e.g. `mainnet`, `xdai`, `arbitrum-one`),
    /// pointing at `https://api.cow.fi/{chain}`.
    pub fn new(chain: &str) -> Self {
        Self { base_url: format!("https://api.cow.fi/{}", chain), client: Client::new() }
    }

    /// Fetches a single order by its unique identifier `uid`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`CowSwapPlacedOrder`].
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

    /// Fetches orders placed by `user`, up to `limit` entries (defaults to 10 when `None`).
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into `Vec<CowSwapPlacedOrder>`.
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

    /// Requests a swap quote for `swap_data`.
    ///
    /// The returned quote's `fee_amount` is zeroed out even though the API returns a non-zero
    /// value.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`CowSwapOrder`].
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

    /// Submits a signed `order` to the Order Book API, returning the created order UID.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
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

/// Parameters for requesting a `CoW` swap quote.
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CowSwapData {
    /// Address of the token being sold.
    pub sell_token: String,
    /// Address of the token being bought.
    pub buy_token: String,
    /// Amount of `sell_token` to sell, before fees, in the token's smallest unit.
    #[serde_as(as = "DisplayFromStr")]
    pub sell_amount_before_fee: U256,
    /// Order kind: `"sell"` or `"buy"`.
    pub kind: String,
    /// Address of the account placing the order.
    pub from: String,
    /// Address that should receive the bought tokens.
    pub receiver: String,
    /// Arbitrary app data string.
    pub app_data: String,
    /// Keccak256 hash of `app_data`.
    pub app_data_hash: String,
}

/// Quote response from the Order Book API, ready to be signed into a [`CowOrder`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CowSwapOrder {
    /// The quoted order parameters.
    pub quote: CowQuote,
    /// Address of the account requesting the quote.
    pub from: String,
    /// Expiration timestamp of the quote (RFC 3339).
    pub expiration: String,
    /// Quote identifier.
    pub id: u64,
    /// Whether the quote has been verified by the API.
    pub verified: bool,
    /// Protocol fee in basis points.
    pub protocol_fee_bps: String,
}

impl CowSwapOrder {
    /// Converts this quote into a [`CowOrder`] ready for submission, attaching `sig` as the
    /// order signature.
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

/// Quoted `CoW` order parameters returned by the quote endpoint.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CowQuote {
    /// Address of the token being sold.
    pub sell_token: String,
    /// Address of the token being bought.
    pub buy_token: String,
    /// Address that should receive the bought tokens, if any.
    pub receiver: Option<String>,
    /// Amount of `sell_token` to sell, in the token's smallest unit.
    #[serde_as(as = "DisplayFromStr")]
    pub sell_amount: U256,
    /// Amount of `buy_token` to receive, in the token's smallest unit.
    #[serde_as(as = "DisplayFromStr")]
    pub buy_amount: U256,
    /// Unix timestamp (seconds) after which the order is no longer valid.
    pub valid_to: u32,
    /// Arbitrary app data string.
    pub app_data: String,
    /// Keccak256 hash of `app_data`.
    pub app_data_hash: String,
    /// Fee amount in `sell_token`, in the token's smallest unit.
    #[serde_as(as = "DisplayFromStr")]
    pub fee_amount: u64,
    /// Order kind: `"sell"` or `"buy"`.
    pub kind: String,
    /// Whether the order may be partially filled.
    pub partially_fillable: bool,
    /// Balance source for the sold token (e.g. `"erc20"`).
    pub sell_token_balance: String,
    /// Balance destination for the bought token (e.g. `"erc20"`).
    pub buy_token_balance: String,
    /// Signing scheme (e.g. `"eip712"`, `"ethsign"`, `"presign"`).
    pub signing_scheme: String,
}

impl CowQuote {
    /// Builds a quote with empty app data, zero fee, `"sell"` kind, `"erc20"` balances, and
    /// `"eip712"` signing scheme; `partially_fillable` is set to `true`.
    pub fn build_quote(
        sell_token: &str,
        buy_token: &str,
        receiver: Option<String>,
        sell_amount: U256,
        buy_amount: U256,
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

    /// Converts this quote into a [`CowOrder`] ready for submission, attaching `sig` as the
    /// order signature and `from` as the ordering account.
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
            from,
        }
    }
}

/// Signed `CoW` order ready for submission to the Order Book API.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CowOrder {
    /// Address of the token being sold.
    pub sell_token: String,
    /// Address of the token being bought.
    pub buy_token: String,
    /// Address that should receive the bought tokens, if any.
    pub receiver: Option<String>,
    /// Amount of `sell_token` to sell, in the token's smallest unit.
    #[serde_as(as = "DisplayFromStr")]
    pub sell_amount: U256,
    /// Amount of `buy_token` to receive, in the token's smallest unit.
    #[serde_as(as = "DisplayFromStr")]
    pub buy_amount: U256,
    /// Unix timestamp (seconds) after which the order is no longer valid.
    pub valid_to: u32,
    /// Arbitrary app data string.
    pub app_data: String,
    /// Keccak256 hash of `app_data`.
    pub app_data_hash: String,
    /// Fee amount in `sell_token`, in the token's smallest unit.
    #[serde_as(as = "DisplayFromStr")]
    pub fee_amount: u64,
    /// Order kind: `"sell"` or `"buy"`.
    pub kind: String,
    /// Whether the order may be partially filled.
    pub partially_fillable: bool,
    /// Balance source for the sold token (e.g. `"erc20"`).
    pub sell_token_balance: String,
    /// Balance destination for the bought token (e.g. `"erc20"`).
    pub buy_token_balance: String,
    /// Signing scheme (e.g. `"eip712"`, `"ethsign"`, `"presign"`).
    pub signing_scheme: String,
    /// Signature over the order.
    pub signature: String,
    /// Address of the account placing the order.
    pub from: String,
}

/// Order record returned by the Order Book API for a placed order.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CowSwapPlacedOrder {
    /// Order creation timestamp (RFC 3339).
    pub creation_date: String,
    /// Address of the order owner.
    pub owner: String,
    /// Unique order identifier.
    pub uid: String,
    /// Balance of the sell token available to the owner, if known.
    pub available_balance: Option<String>,
    /// Total amount of `buy_token` executed so far, in the token's smallest unit.
    pub executed_buy_amount: String,
    /// Total amount of `sell_token` executed so far, in the token's smallest unit.
    pub executed_sell_amount: String,
    /// Total amount of `sell_token` executed before fees, in the token's smallest unit.
    pub executed_sell_amount_before_fees: String,
    /// Total fee amount executed so far, in the token's smallest unit.
    pub executed_fee_amount: String,
    /// Total fee executed so far.
    pub executed_fee: String,
    /// Address of the token the fee was paid in.
    pub executed_fee_token: String,
    /// Whether the order has been invalidated.
    pub invalidated: bool,
    /// Order status (e.g. `"open"`, `"fulfilled"`, `"cancelled"`, `"expired"`).
    pub status: String,
    /// Order class (e.g. `"market"`, `"limit"`, `"liquidity"`).
    pub class: String,
    /// Address of the settlement contract.
    pub settlement_contract: String,
    /// Whether this order is a liquidity order.
    pub is_liquidity_order: bool,
    /// Full app data JSON string.
    pub full_app_data: String,
    /// Quote associated with the order, if any.
    pub quote: Option<CowSwapExistingQuote>,
    /// Address of the token being sold.
    pub sell_token: String,
    /// Address of the token being bought.
    pub buy_token: String,
    /// Address that receives the bought tokens.
    pub receiver: String,
    /// Amount of `sell_token` to sell, in the token's smallest unit.
    pub sell_amount: String,
    /// Amount of `buy_token` to receive, in the token's smallest unit.
    pub buy_amount: String,
    /// Unix timestamp (seconds) after which the order is no longer valid.
    pub valid_to: u64,
    /// Arbitrary app data string.
    pub app_data: String,
    /// Fee amount in `sell_token`, in the token's smallest unit.
    pub fee_amount: String,
    /// Order kind: `"sell"` or `"buy"`.
    pub kind: String,
    /// Whether the order may be partially filled.
    pub partially_fillable: bool,
    /// Balance source for the sold token (e.g. `"erc20"`).
    pub sell_token_balance: String,
    /// Balance destination for the bought token (e.g. `"erc20"`).
    pub buy_token_balance: String,
    /// Signing scheme (e.g. `"eip712"`, `"ethsign"`, `"presign"`).
    pub signing_scheme: String,
    /// Signature over the order.
    pub signature: String,
    /// Solver interactions attached to the order.
    pub interactions: Interactions,
}

/// Quote details attached to an existing placed order.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CowSwapExistingQuote {
    /// Estimated gas amount for settlement.
    pub gas_amount: String,
    /// Estimated gas price.
    pub gas_price: String,
    /// Price of `sell_token` in `buy_token`.
    pub sell_token_price: String,
    /// Quoted sell amount, in the token's smallest unit.
    pub sell_amount: String,
    /// Quoted buy amount, in the token's smallest unit.
    pub buy_amount: String,
    /// Quoted fee amount, in the token's smallest unit.
    pub fee_amount: String,
    /// Address of the solver that produced the quote.
    pub solver: String,
    /// Whether the quote has been verified.
    pub verified: bool,
}

/// Solver interactions attached to an order.
#[derive(Debug, Clone, Deserialize)]
pub struct Interactions {
    /// Interactions to run before the order is settled.
    pub pre: Vec<String>,
    /// Interactions to run after the order is settled.
    pub post: Vec<String>,
}
