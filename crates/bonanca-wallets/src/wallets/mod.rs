//! Chain-specific wallet implementations.
//!
//! [`evm::EvmWallet`] targets Ethereum-compatible chains and [`solana::SolWallet`] targets
//! Solana. Both follow the same view/load split described in the crate documentation.

pub mod evm;
pub mod solana;
