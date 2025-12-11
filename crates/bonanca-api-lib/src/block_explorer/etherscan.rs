use core::panic;

use anyhow::Result;
use bonanca_core::cashflows::{NativeFlow, TokenFlow};
use reqwest::Client;
use serde::Deserialize;

pub struct EtherscanApi {
    pub base_url: String,
    pub api_key: String,
}

impl EtherscanApi {
    pub fn new(api_key: String) -> Self {
        Self {
            base_url: "https://api.etherscan.io/v2/api".to_string(),
            api_key,
        }
    }

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

        match resp.result {
            EtherscanResult::Native(res) => Ok(res),
            EtherscanResult::Token(_) => panic!(),
        }
    }

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

        match resp.result {
            EtherscanResult::Native(_) => panic!(),
            EtherscanResult::Token(res) => Ok(res),
        }
    }
}

impl From<EtherscanTransaction> for NativeFlow {
    fn from(trans: EtherscanTransaction) -> Self {
        let big_value: f64 = trans.value.parse().unwrap();
        let big_gas: f64 = trans.gas_used.parse().unwrap();
        let value = big_value / 1e18;
        let gas_used = big_gas / 1e18;

        NativeFlow {
            block: trans.block_number.parse().unwrap(),
            timestamp: trans.time_stamp,
            to: trans.to,
            from: trans.from,
            value,
            gas_used,
        }
    }
}

impl From<EtherscanTokenTransaction> for TokenFlow {
    fn from(trans: EtherscanTokenTransaction) -> Self {
        let big_value: f64 = trans.value.parse().unwrap();
        let big_gas: f64 = trans.gas_used.parse().unwrap();
        let decimal: i32 = trans.token_decimal.parse().unwrap();
        let value = big_value / 10.0_f64.powi(decimal);
        let gas_used = big_gas / 1e18;

        TokenFlow {
            block: trans.block_number.parse().unwrap(),
            timestamp: trans.time_stamp,
            token: trans.token_symbol,
            to: trans.to,
            from: trans.from,
            value,
            gas_used,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EtherscanResponse {
    pub status: String,
    pub message: String,
    pub result: EtherscanResult,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EtherscanResult {
    Native(Vec<EtherscanTransaction>),
    Token(Vec<EtherscanTokenTransaction>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtherscanTransaction {
    pub block_number: String,
    pub time_stamp: String,
    pub hash: String,
    pub nonce: String,
    pub block_hash: String,
    pub transaction_index: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: String,
    pub gas_price: String,
    pub is_error: String,
    #[serde(rename = "txreceipt_status")]
    pub txreceipt_status: String,
    pub input: String,
    pub contract_address: String,
    pub cumulative_gas_used: String,
    pub gas_used: String,
    pub confirmations: String,
    pub method_id: String,
    pub function_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtherscanTokenTransaction {
    pub block_number: String,
    pub time_stamp: String,
    pub hash: String,
    pub nonce: String,
    pub block_hash: String,
    pub from: String,
    pub contract_address: String,
    pub to: String,
    pub value: String,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimal: String,
    pub transaction_index: String,
    pub gas: String,
    pub gas_price: String,
    pub gas_used: String,
    pub cumulative_gas_used: String,
    pub input: String,
    pub method_id: String,
    pub function_name: String,
    pub confirmations: String,
}
