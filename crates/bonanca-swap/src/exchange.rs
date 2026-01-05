use anyhow::Result;
use async_trait::async_trait;
use bonanca_api_lib::defi::{jupiter::Jupiter, zerox::ZeroX};
use bonanca_wallets::{CryptoWallets, TransactionData};

#[async_trait]
pub trait Exchange {
    async fn get_swap_data(
        &self,
        wallet_enum: &CryptoWallets,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> Result<TransactionData>;
}

pub fn get_exchange(
    name: &str,
    api_key: String,
    chain_id: Option<u16>,
) -> Result<Box<dyn Exchange>> {
    let exchange: Box<dyn Exchange> = match name {
        "0x" => Box::new(ZeroX::new(api_key, chain_id.unwrap())),
        "Jupiter" => Box::new(Jupiter::new(api_key)),
        _ => Err(anyhow::anyhow!("Unsupported aggregator"))?,
    };

    Ok(exchange)
}
