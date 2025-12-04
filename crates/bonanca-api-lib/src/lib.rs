pub mod defi;
pub mod lending_oracle;

use anyhow::Result;
use bonanca_core::traits::{Exchange, Oracle};
use defi::{cmc::CoinMarketCap, defi_llama::DefiLlama, jupiter::Jupiter, zerox::ZeroX};

pub fn get_oracle(name: &str, api_url: &str, api_key: &Option<String>) -> Result<Box<dyn Oracle>> {
    let oracle: Box<dyn Oracle> = match name {
        "CoinMarketCap" => Box::new(CoinMarketCap::new(api_url.to_string(), api_key.clone())),
        "Jupiter" => Box::new(Jupiter::new(api_url.to_string(), api_key.clone())),
        "DefiLlama" => Box::new(DefiLlama::new(api_url.to_string())),
        _ => Err(anyhow::anyhow!("Unsupported oracle"))?,
    };

    Ok(oracle)
}

pub fn get_exchange(
    name: &str,
    api_url: &str,
    api_key: &Option<String>,
    chain_id: Option<u16>,
) -> Result<Box<dyn Exchange>> {
    let exchange: Box<dyn Exchange> = match name {
        "0x" => Box::new(ZeroX::new(
            api_url.to_string(),
            api_key.clone(),
            chain_id.unwrap(),
        )),
        "Jupiter" => Box::new(Jupiter::new(api_url.to_string(), api_key.clone())),
        _ => Err(anyhow::anyhow!("Unsupported aggregator"))?,
    };

    Ok(exchange)
}
