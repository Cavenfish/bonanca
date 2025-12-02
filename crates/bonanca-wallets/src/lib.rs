mod wallets;

use anyhow::Result;
use bonanca_core::traits::Wallet;
use std::path::Path;
use wallets::{evm::EvmWallet, solana::SolWallet};

pub fn get_wallet(
    chain: &str,
    keyvault: &Path,
    rpc_url: &str,
    child: u32,
) -> Result<Box<dyn Wallet + Send + Sync>> {
    let wallet: Box<dyn Wallet + Send + Sync> = match chain.split(":").next().unwrap() {
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
    let wallet: Box<dyn Wallet + Send + Sync> = match chain.split(":").next().unwrap() {
        "EVM" => Box::new(EvmWallet::view(keyvault, rpc_url, child)),
        "Solana" => Box::new(SolWallet::view(keyvault, rpc_url, child)),
        _ => Err(anyhow::anyhow!("Unsupported chain"))?,
    };

    Ok(wallet)
}
