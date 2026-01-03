use alloy::{
    network::TransactionBuilder,
    rpc::types::{TransactionInput, TransactionRequest},
};
use alloy_primitives::{Address, Bytes, Uint, hex::decode};
use anyhow::Result;
use async_trait::async_trait;
use bonanca_api_lib::defi::zerox::ZeroX;
use bonanca_wallets::{TransactionData, Wallet};
use std::str::FromStr;

use crate::exchange::Exchange;

#[async_trait]
impl Exchange for ZeroX {
    async fn get_swap_data(
        &self,
        wallet: &Box<dyn Wallet + Send + Sync>,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> Result<TransactionData> {
        let taker = wallet.get_pubkey()?;

        let big_amount = wallet.parse_token_amount(amount, sell).await?;

        let quote = self.get_swap_quote(sell, buy, big_amount, &taker).await?;

        // if let Some(issues) = quote.issues.allowance {
        //     let tmp = wallet
        //         .check_swap(sell, amount, Some(&issues.spender))
        //         .await?;

        //     if !tmp {
        //         std::process::exit(1)
        //     };
        // };

        let taker_addy = Address::from_str(&taker)?;
        let to_addy = Address::from_str(&quote.transaction.to)?;
        let tmp = decode(quote.transaction.data)?;
        let data = Bytes::copy_from_slice(&tmp);
        let value: Uint<256, 4> = quote.transaction.value.parse()?;
        let gas_limit: u64 = quote.transaction.gas.parse()?;
        let gas_price: u128 = quote.transaction.gas_price.parse()?;

        let input = TransactionInput::new(data);

        let tx = TransactionRequest::default()
            .input(input)
            .with_input_and_data()
            .with_from(taker_addy)
            .with_to(to_addy)
            .with_value(value)
            .with_gas_limit(gas_limit)
            .with_max_fee_per_gas(gas_price);

        Ok(TransactionData::Evm(tx))
    }
}
