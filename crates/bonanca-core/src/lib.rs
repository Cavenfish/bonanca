pub mod api_lib;
pub mod config;
pub mod holdings;
pub mod wallets;

use anyhow::Result;
use api_lib::{
    cmc::CoinMarketCap,
    jupiter::Jupiter,
    traits::{Exchange, Oracle},
    zerox::ZeroX,
};
use dirs::data_dir;
use std::{fs::create_dir_all, path::Path};
use wallets::{evm::EvmWallet, solana::SolWallet, traits::Wallet};

use crate::api_lib::defi_llama::DefiLlama;
use crate::config::Config;

pub fn init_config() {
    let config_dir = data_dir().unwrap().join("bonanca");

    if !config_dir.exists() {
        create_dir_all(&config_dir).unwrap();
    }

    let config_file = config_dir.join("config.json");

    if !config_file.exists() {
        let config = Config::default();

        config.write();
    }
}

pub fn get_wallet(
    chain: &str,
    keyvault: &Path,
    rpc_url: &str,
    child: u32,
) -> Result<Box<dyn Wallet + Send + Sync>> {
    let wallet: Box<dyn Wallet + Send + Sync> = match chain {
        "EVM" => Box::new(EvmWallet::load(keyvault, rpc_url, child)),
        "Solana" => Box::new(SolWallet::load(keyvault, rpc_url, child)),
        _ => Err(anyhow::anyhow!("Unsupported chain"))?,
    };

    Ok(wallet)
}

pub fn get_wallet_view(
    chain: &str,
    keyvault: &Path,
    rpc_url: &str,
    child: u32,
) -> Result<Box<dyn Wallet + Send + Sync>> {
    let wallet: Box<dyn Wallet + Send + Sync> = match chain {
        "EVM" => Box::new(EvmWallet::view(keyvault, rpc_url, child)),
        "Solana" => Box::new(SolWallet::view(keyvault, rpc_url, child)),
        _ => Err(anyhow::anyhow!("Unsupported chain"))?,
    };

    Ok(wallet)
}

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
