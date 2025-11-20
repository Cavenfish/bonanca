pub mod api_lib;
pub mod holdings;
pub mod wallets;

use anyhow::Result;
use api_lib::{
    cmc::CoinMarketCap,
    jupiter::Jupiter,
    traits::{Exchange, Oracle},
    zerox::ZeroX,
};
use std::path::Path;
use wallets::{evm::EvmWallet, solana::SolWallet, traits::Wallet};

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

pub fn get_oracle(name: &str, api_url: &str, api_key: &str) -> Result<Box<dyn Oracle>> {
    let oracle: Box<dyn Oracle> = match name {
        "CoinMarketCap" => Box::new(CoinMarketCap::new(api_url.to_string(), api_key.to_string())),
        "Jupiter" => Box::new(Jupiter::new(api_url.to_string(), api_key.to_string())),
        _ => Err(anyhow::anyhow!("Unsupported oracle"))?,
    };

    Ok(oracle)
}

pub fn get_exchange(
    name: &str,
    api_url: &str,
    api_key: &str,
    chain_id: Option<u16>,
) -> Result<Box<dyn Exchange>> {
    let exchange: Box<dyn Exchange> = match name {
        "0x" => Box::new(ZeroX::new(
            api_url.to_string(),
            api_key.to_string(),
            chain_id.unwrap(),
        )),
        "Jupiter" => Box::new(Jupiter::new(api_url.to_string(), api_key.to_string())),
        _ => Err(anyhow::anyhow!("Unsupported aggregator"))?,
    };

    Ok(exchange)
}
