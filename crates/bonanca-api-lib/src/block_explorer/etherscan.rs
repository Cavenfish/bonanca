use core::panic;

use anyhow::Result;
use bonanca_core::transactions::{CryptoOperation, CryptoTransfer, EvmApprove, Txn};
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
    ) -> Result<Vec<(String, Txn)>> {
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

        let txns = results
            .into_iter()
            .map(|r| (r.hash.clone(), r.make_txn(pubkey).unwrap()))
            .collect();

        Ok(txns)
    }

    pub async fn get_token_history(
        &self,
        chain_id: u64,
        pubkey: &str,
        start_block: u64,
    ) -> Result<Vec<(String, Txn)>> {
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

        let txns = results
            .into_iter()
            .map(|r| (r.hash.clone(), r.make_txn(pubkey).unwrap()))
            .collect();

        Ok(txns)
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

impl EtherscanTransaction {
    pub fn make_txn(self, pubkey: &str) -> Result<Txn> {
        let big_value: f64 = self.value.parse()?;
        let big_gas: f64 = self.gas_used.parse()?;
        let gas_price: f64 = self.gas_price.parse()?;
        let gas_used: f64 = (big_gas * gas_price) / 1e18;
        let amount = big_value / 1e18;

        let operation: CryptoOperation = if self.method_id.as_str() == "0x" {
            CryptoOperation::Transfer(CryptoTransfer {
                token: "Native".to_string(),
                amount,
                from: self.from,
                to: self.to,
            })
        } else if self.function_name.as_str() == "approve(address spender, uint256 rawAmount)" {
            CryptoOperation::Approve(EvmApprove { token: self.to })
        } else {
            CryptoOperation::None
        };

        let txn = Txn {
            pubkey: pubkey.to_string(),
            block: self.block_number.parse()?,
            timestamp: self.time_stamp.parse()?,
            gas_used,
            operation,
        };

        Ok(txn)
    }
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

impl EtherscanTokenTransaction {
    pub fn make_txn(self, pubkey: &str) -> Result<Txn> {
        let big_value: f64 = self.value.parse()?;
        let big_gas: f64 = self.gas_used.parse()?;
        let gas_price: f64 = self.gas_price.parse()?;
        let decimal: i32 = self.token_decimal.parse()?;
        let gas_used: f64 = (big_gas * gas_price) / 1e18;
        let amount = big_value / 10.0_f64.powi(decimal);

        // TODO
        let operation: CryptoOperation =
            if self.function_name.as_str() == "transfer(address dst, uint256 rawAmount)" {
                CryptoOperation::Transfer(CryptoTransfer {
                    token: self.token_symbol,
                    amount,
                    from: self.from,
                    to: self.to,
                })
            } else {
                CryptoOperation::None
            };

        let txn = Txn {
            pubkey: pubkey.to_string(),
            block: self.block_number.parse()?,
            timestamp: self.time_stamp.parse()?,
            gas_used,
            operation,
        };

        Ok(txn)
    }
}
