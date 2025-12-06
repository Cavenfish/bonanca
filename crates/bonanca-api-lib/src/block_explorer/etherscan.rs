use anyhow::Result;
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

    pub async fn get_account_history(
        &self,
        chain_id: u32,
        pubkey: &str,
        start_block: u64,
    ) -> Result<EtherscanResponse> {
        let client = Client::new();
        let url = format!(
            "{}?apiKey={}?chainid={}?address={}?startblock={}",
            &self.base_url, &self.api_key, chain_id, pubkey, start_block
        );

        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<EtherscanResponse>()
            .await?;

        Ok(resp)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EtherscanResponse {
    pub status: String,
    pub message: String,
    pub result: Vec<Transaction>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
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
    pub txreceipt_status: String,
    pub input: String,
    pub contract_address: String,
    pub cumulative_gas_used: String,
    pub gas_used: String,
    pub confirmations: String,
    pub method_id: String,
    pub function_name: String,
}
