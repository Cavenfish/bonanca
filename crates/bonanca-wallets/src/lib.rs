pub mod wallets;

use alloy::rpc::types::TransactionRequest;
use anyhow::Result;
use solana_sdk::transaction::VersionedTransaction;

pub enum TransactionData {
    Evm(TransactionRequest),
    Sol(VersionedTransaction),
}

pub trait WalletView<T, U> {
    fn view(value: T, rpc: U) -> Self;
}

pub trait WalletLoad<T, U> {
    fn load(value: T, rpc: U) -> Self;
}

pub trait HdWalletView<T, U> {
    fn view(value: T, rpc: U, child: u32) -> Self;
}

pub trait HdWalletLoad<T, U> {
    fn load(value: T, rpc: U, child: u32) -> Self;
}

pub trait HdWallets<T> {
    fn get_child_keypair(&self, child: u32) -> Result<T>;
}
