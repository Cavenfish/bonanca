//! Multi-chain wallet abstractions for EVM and Solana.
//!
//! This crate provides unified view and loaded wallet implementations for
//! interacting with Ethereum-compatible blockchains and Solana:
//!
//! - [`wallets::evm::EvmWallet`]: EVM wallet supporting balance queries, ERC-20 token interactions,
//!   gas estimation, native/token transfers, and raw signing.
//! - [`wallets::solana::SolWallet`]: Solana wallet supporting balance queries, SPL token account
//!   management, native/token transfers, token burning, and transaction sending.
//!
//! Wallets can be created in read-only / view-only mode via [`WalletView`] and
//! [`HdWalletView`], or in full transacting mode with an active signer via
//! [`WalletLoad`] and [`HdWalletLoad`].
//!
//! # Errors
//!
//! Error handling is split into specialized enums and unified under a top-level enum:
//!
//! - [`SharedWalletErrors`]: Common failures shared across chains (keyvault operations, key
//!   derivation, missing signers on view-only wallets, RPC URL parsing, and float conversions).
//! - [`EvmWalletErrors`]: Failures specific to EVM wallets (address parsing, unit conversion, RPC
//!   transport, ERC-20 contract calls, transaction receipts, and EVM signing).
//! - [`SolWalletErrors`]: Failures specific to Solana wallets (public key parsing, Solana client
//!   RPC errors, SPL token account lookups, transaction receipts, and Solana signing).
//! - [`WalletErrors`]: Top-level unified enum that wraps the above three via transparent
//!   conversions, used across shared wallet traits.

pub mod wallets;

use std::num::ParseFloatError;

use alloy::rpc::types::TransactionRequest;
use bonanca_keyvault::KeyVaultErrors;
use solana_sdk::transaction::VersionedTransaction;
use thiserror::Error;
pub use wallets::{evm::EvmWalletErrors, solana::SolWalletErrors};

/// Errors common to both EVM and Solana wallet operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum GeneralWalletErrors {
    /// KeyVault-related errors arising during vault loading, decryption,
    /// or HD child key derivation.
    #[error("KeyVaultError: {0}")]
    KeyVault(#[from] KeyVaultErrors),

    /// No public key was recorded in the keyvault for the requested derivation path.
    #[error("no public key found in keyvault for derivation path: {0}")]
    MissingPublicKey(String),

    /// The provided private key bytes are invalid or cannot be parsed into a signing key.
    #[error("invalid private key")]
    InvalidPrivateKey,

    /// An operation requiring a transaction signature or private key was attempted
    /// on a wallet loaded in view-only mode.
    #[error("wallet is in view-only mode: operation requires a private key / signer")]
    MissingSigner,

    /// The provided RPC endpoint URL could not be parsed.
    #[error("invalid RPC URL: {0}")]
    InvalidRpcUrl(String),

    /// Parsing a numeric string into a floating-point value failed.
    #[error("failed to parse floating-point number: {0}")]
    ParseFloat(#[from] ParseFloatError),
}

/// Top-level unified error type for all wallet operations across EVM and Solana.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum WalletErrors {
    /// Errors common to all wallets (keys, vault, signer status, URL parsing).
    #[error(transparent)]
    General(#[from] GeneralWalletErrors),

    /// EVM-specific wallet errors.
    #[error(transparent)]
    Evm(#[from] EvmWalletErrors),

    /// Solana-specific wallet errors.
    #[error(transparent)]
    Sol(#[from] SolWalletErrors),
}

impl From<KeyVaultErrors> for WalletErrors {
    fn from(err: KeyVaultErrors) -> Self {
        Self::General(GeneralWalletErrors::KeyVault(err))
    }
}

pub enum TransactionData {
    Evm(TransactionRequest),
    Sol(VersionedTransaction),
}

pub trait WalletView<T>: Sized {
    fn view(value: T, rpc: &str) -> Result<Self, WalletErrors>;
}

pub trait WalletLoad<T>: Sized {
    fn load(value: T, rpc: &str) -> Result<Self, WalletErrors>;
}

pub trait HdWalletView<T, U>: Sized {
    fn view(value: T, rpc: &str, child: U) -> Result<Self, WalletErrors>;
}

pub trait HdWalletLoad<T, U>: Sized {
    fn load(value: T, rpc: &str, child: U, password: &str) -> Result<Self, WalletErrors>;
}

pub(crate) trait HdWallets<T, U> {
    fn get_child_keypair(&self, child: U) -> Result<T, WalletErrors>;
}
