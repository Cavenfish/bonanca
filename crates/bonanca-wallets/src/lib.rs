pub mod wallets;

use alloy::rpc::types::TransactionRequest;
use anyhow::Result;
use solana_sdk::transaction::VersionedTransaction;

pub enum TransactionData {
    Evm(TransactionRequest),
    Sol(VersionedTransaction),
}

pub trait WalletView<T> {
    fn view(value: T, rpc: &str) -> Self;
}

pub trait WalletLoad<T> {
    fn load(value: T, rpc: &str) -> Self;
}

pub trait HdWalletView<T, U> {
    fn view(value: T, rpc: &str, child: U) -> Self;
}

pub trait HdWalletLoad<T, U> {
    fn load(value: T, rpc: &str, child: U) -> Self;
}

pub trait HdWallets<T, U> {
    fn get_child_keypair(&self, child: U) -> Result<T>;
}
