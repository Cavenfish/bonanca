pub mod block_explorer;
pub mod defi;
pub mod lending_oracle;

use anyhow::Result;
use bonanca_core::traits::{Exchange, Oracle};
use defi::{cmc::CoinMarketCap, defi_llama::DefiLlama, jupiter::Jupiter, zerox::ZeroX};

pub fn get_oracle(name: &str, api_key: String) -> Result<Box<dyn Oracle>> {
    let oracle: Box<dyn Oracle> = match name {
        "CoinMarketCap" => Box::new(CoinMarketCap::new(api_key)),
        "Jupiter" => Box::new(Jupiter::new(api_key)),
        "DefiLlama" => Box::new(DefiLlama::new()),
        _ => Err(anyhow::anyhow!("Unsupported oracle"))?,
    };

    Ok(oracle)
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
