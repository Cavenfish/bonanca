//! Etherscan V2 API client and transaction history response types.

use core::panic;

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

/// Client for the Etherscan V2 account transaction history endpoints.
pub struct EtherscanApi {
    /// Base URL for the Etherscan V2 API endpoint.
    pub base_url: String,
    /// Etherscan API key appended to requests.
    pub api_key: String,
}

impl EtherscanApi {
    /// Creates a new [`EtherscanApi`] client authenticated with `api_key`.
    pub fn new(api_key: String) -> Self {
        Self { base_url: "https://api.etherscan.io/v2/api".to_string(), api_key }
    }

    /// Fetches native transaction history for `pubkey` on `chain_id` starting at `start_block`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`EtherscanResponse`] (for example, when Etherscan returns an error message string).
    ///
    /// # Panics
    ///
    /// Panics if the deserialized response contains [`EtherscanResult::Token`] instead of
    /// [`EtherscanResult::Native`].
    pub async fn get_native_history(
        &self,
        chain_id: u64,
        pubkey: &str,
        start_block: u64,
    ) -> Result<Vec<EtherscanTransaction>> {
        let client = Client::new();
        let url = format!(
            "{}?apiKey={}&chainid={}&address={}&startblock={}&module=account&action=txlist",
            &self.base_url, &self.api_key, chain_id, pubkey, start_block
        );

        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<EtherscanResponse>()
            .await?;

        let results = match resp.result {
            EtherscanResult::Native(res) => res,
            EtherscanResult::Token(_) => panic!(),
        };

        Ok(results)
    }

    /// Fetches ERC-20 token transfer history for `pubkey` on `chain_id` starting at
    /// `start_block`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`EtherscanResponse`] (for example, when Etherscan returns an error message string).
    ///
    /// # Panics
    ///
    /// Panics if the deserialized response contains [`EtherscanResult::Native`] instead of
    /// [`EtherscanResult::Token`]. Because [`EtherscanResult`] is `#[serde(untagged)]` with
    /// `Native` listed first, an empty JSON array (`[]`) deserializes as `Native` and will also
    /// panic.
    pub async fn get_token_history(
        &self,
        chain_id: u64,
        pubkey: &str,
        start_block: u64,
    ) -> Result<Vec<EtherscanTokenTransaction>> {
        let client = Client::new();
        let url = format!(
            "{}?apiKey={}&chainid={}&address={}&startblock={}&module=account&action=tokentx",
            &self.base_url, &self.api_key, chain_id, pubkey, start_block
        );

        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<EtherscanResponse>()
            .await?;

        let results = match resp.result {
            EtherscanResult::Native(_) => panic!(),
            EtherscanResult::Token(res) => res,
        };

        Ok(results)
    }
}

/// Top-level response envelope returned by Etherscan V2 endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct EtherscanResponse {
    /// Status code string (`"1"` for success, `"0"` for error or empty result).
    pub status: String,
    /// Human-readable status message (e.g. `"OK"`).
    pub message: String,
    /// Deserialized list of native or token transactions.
    pub result: EtherscanResult,
}

/// Untagged result payload variants returned by Etherscan V2 account endpoints.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EtherscanResult {
    /// List of native currency transactions (`action=txlist`).
    Native(Vec<EtherscanTransaction>),
    /// List of ERC-20 token transfer transactions (`action=tokentx`).
    Token(Vec<EtherscanTokenTransaction>),
}

/// Native transaction entry returned by Etherscan's `txlist` endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtherscanTransaction {
    /// Block number containing the transaction.
    pub block_number: String,
    /// Unix timestamp when the block was mined.
    pub time_stamp: String,
    /// Transaction hash.
    pub hash: String,
    /// Sender account nonce.
    pub nonce: String,
    /// Hash of the containing block.
    pub block_hash: String,
    /// Index of the transaction within the block.
    pub transaction_index: String,
    /// Sender address.
    pub from: String,
    /// Recipient or contract address.
    pub to: String,
    /// Transferred native value in wei.
    pub value: String,
    /// Gas limit allocated for the transaction.
    pub gas: String,
    /// Gas price in wei.
    pub gas_price: String,
    /// Execution error flag (`"0"` if successful, `"1"` if reverted).
    pub is_error: String,
    /// Transaction receipt status (`"1"` for success, `"0"` for failure).
    #[serde(rename = "txreceipt_status")]
    pub txreceipt_status: String,
    /// Hex-encoded calldata input.
    pub input: String,
    /// Deployed contract address if contract creation, otherwise empty.
    pub contract_address: String,
    /// Cumulative gas used in the block up to this transaction.
    pub cumulative_gas_used: String,
    /// Gas consumed by this transaction.
    pub gas_used: String,
    /// Number of block confirmations.
    pub confirmations: String,
    /// Four-byte function selector if applicable.
    pub method_id: String,
    /// Decoded function signature if available.
    pub function_name: String,
}

/// ERC-20 token transfer entry returned by Etherscan's `tokentx` endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtherscanTokenTransaction {
    /// Block number containing the transfer.
    pub block_number: String,
    /// Unix timestamp when the block was mined.
    pub time_stamp: String,
    /// Transaction hash.
    pub hash: String,
    /// Sender account nonce.
    pub nonce: String,
    /// Hash of the containing block.
    pub block_hash: String,
    /// Token sender address.
    pub from: String,
    /// Token contract address.
    pub contract_address: String,
    /// Token recipient address.
    pub to: String,
    /// Raw token transfer amount in base units.
    pub value: String,
    /// Token name.
    pub token_name: String,
    /// Token ticker symbol.
    pub token_symbol: String,
    /// Number of decimals used by the token.
    pub token_decimal: String,
    /// Index of the transaction within the block.
    pub transaction_index: String,
    /// Gas limit allocated for the transaction.
    pub gas: String,
    /// Gas price in wei.
    pub gas_price: String,
    /// Gas consumed by this transaction.
    pub gas_used: String,
    /// Cumulative gas used in the block up to this transaction.
    pub cumulative_gas_used: String,
    /// Hex-encoded calldata input.
    pub input: String,
    /// Four-byte function selector if applicable.
    pub method_id: String,
    /// Decoded function signature if available.
    pub function_name: String,
    /// Number of block confirmations.
    pub confirmations: String,
}
