pub mod wallets;

use alloy::rpc::types::TransactionRequest;
use anyhow::Result;
use solana_sdk::transaction::VersionedTransaction;

pub enum TransactionData {
    Evm(TransactionRequest),
    Sol(VersionedTransaction),
}

pub trait WalletView<T>: Sized {
    fn view(value: T, rpc: &str) -> Result<Self>;
}

pub trait WalletLoad<T>: Sized {
    fn load(value: T, rpc: &str) -> Result<Self>;
}

pub trait HdWalletView<T, U>: Sized {
    fn view(value: T, rpc: &str, child: U) -> Result<Self>;
}

pub trait HdWalletLoad<T, U>: Sized {
    fn load(value: T, rpc: &str, child: U, password: &str) -> Result<Self>;
}

pub(crate) trait HdWallets<T, U> {
    fn get_child_keypair(&self, child: U) -> Result<T>;
}
