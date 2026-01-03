mod wallets;

use alloy::rpc::types::TransactionRequest;
use alloy::signers::{k256::ecdsa::SigningKey, local::LocalSigner};
use anyhow::Result;
use async_trait::async_trait;
use bonanca_db::transactions::Txn;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::VersionedTransaction;
use std::path::Path;

use wallets::{evm::EvmWallet, solana::SolWallet};

pub enum TransactionData {
    Sol(VersionedTransaction),
    Evm(TransactionRequest),
}

pub enum CryptoSigners {
    Evm(LocalSigner<SigningKey>),
    Sol(Keypair),
}

#[async_trait]
pub trait Wallet {
    fn get_pubkey(&self) -> Result<String>;

    fn get_signer(&self) -> Result<CryptoSigners>;

    fn parse_native_amount(&self, amount: f64) -> Result<u64>;

    async fn parse_token_amount(&self, amount: f64, token: &str) -> Result<u64>;

    async fn close(&self, to: &str) -> Result<()>;

    // async fn get_history(&self) -> Result<Vec<(String, Txn)>>;

    async fn balance(&self) -> Result<f64>;

    async fn transfer(&self, to: &str, amount: f64) -> Result<(String, Txn)>;

    async fn token_balance(&self, token: &str) -> Result<f64>;

    async fn transfer_token(&self, token: &str, amount: f64, to: &str) -> Result<(String, Txn)>;

    async fn transfer_all_tokens(&self, token: &str, to: &str) -> Result<()>;

    async fn sign_and_send(&self, txn: TransactionData) -> Result<()>;
}

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
