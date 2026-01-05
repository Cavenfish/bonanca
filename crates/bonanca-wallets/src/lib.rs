pub mod wallets;

use alloy::rpc::types::TransactionRequest;
use solana_sdk::transaction::VersionedTransaction;

use wallets::{evm::EvmWallet, solana::SolWallet};

pub enum TransactionData {
    Evm(TransactionRequest),
    Sol(VersionedTransaction),
}

pub enum CryptoWallets {
    Evm(EvmWallet),
    Sol(SolWallet),
}
