pub mod block_explorer;
pub mod defi;
pub mod lending_oracle;

use anyhow::Result;
use async_trait::async_trait;
use defi::{cmc::CoinMarketCap, defi_llama::DefiLlama, jupiter::Jupiter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Asset {
    pub name: String,
    pub symbol: String,
    pub address: String,
}

#[async_trait]
pub trait Oracle {
    async fn get_token_value(&self, asset: &Asset, amount: f64, chain: &str) -> Result<f64>;
}

pub fn get_oracle(name: &str, api_key: String) -> Result<Box<dyn Oracle>> {
    let oracle: Box<dyn Oracle> = match name {
        "CoinMarketCap" => Box::new(CoinMarketCap::new(api_key)),
        "Jupiter" => Box::new(Jupiter::new(api_key)),
        "DefiLlama" => Box::new(DefiLlama::new()),
        _ => Err(anyhow::anyhow!("Unsupported oracle"))?,
    };

    Ok(oracle)
}
