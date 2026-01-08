use alloy::{
    network::TransactionBuilder,
    rpc::types::{TransactionInput, TransactionRequest},
};
use alloy_primitives::{Address, Bytes, Uint, hex::decode};
use anyhow::Result;
use bonanca_api_lib::defi::zerox::{Issues, ZeroXApi};
use bonanca_wallets::wallets::evm::EvmWallet;
use std::str::FromStr;

pub struct ZeroX {
    api: ZeroXApi,
}

impl ZeroX {
    pub fn new(api_key: String, chain_id: u16) -> Self {
        let api = ZeroXApi::new(api_key, chain_id);
        Self { api }
    }

    pub async fn check_swap(
        &self,
        wallet: &EvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> Result<Issues> {
        let taker = wallet.get_pubkey()?;

        let big_amount = wallet.parse_token_amount(amount, sell).await?;

        let quote = self
            .api
            .get_swap_quote(sell, buy, big_amount, &taker)
            .await?;

        Ok(quote.issues)
    }

    pub async fn swap(&self, wallet: &EvmWallet, sell: &str, buy: &str, amount: f64) -> Result<()> {
        let taker = wallet.get_pubkey()?;

        let big_amount = wallet.parse_token_amount(amount, sell).await?;

        let quote = self
            .api
            .get_swap_quote(sell, buy, big_amount, &taker)
            .await?;

        if let Some(issues) = quote.issues.allowance {
            panic!("Allowance issues: {:?}", issues);
        };

        if let Some(issues) = quote.issues.balance {
            panic!("Balance issues: {:?}", issues);
        };

        let taker_addy = Address::from_str(&taker)?;
        let to_addy = Address::from_str(&quote.transaction.to)?;
        let tmp = decode(quote.transaction.data)?;
        let data = Bytes::copy_from_slice(&tmp);
        let value: Uint<256, 4> = quote.transaction.value.parse()?;
        let gas_limit: u64 = quote.transaction.gas.parse()?;
        let gas_price: u128 = quote.transaction.gas_price.parse()?;

        let input = TransactionInput::new(data);

        let txn = TransactionRequest::default()
            .input(input)
            .with_input_and_data()
            .with_from(taker_addy)
            .with_to(to_addy)
            .with_value(value)
            .with_gas_limit(gas_limit)
            .with_max_fee_per_gas(gas_price);

        let _ = wallet.sign_and_send(txn).await.unwrap();

        Ok(())
    }
}
