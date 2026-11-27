//! Decentralized finance (DeFi) protocol API clients.

/// Aave V3 GraphQL API client and reserve types.
pub mod aave;
/// `CoW` Protocol Order Book API client and order types.
pub mod cow;
/// Jupiter API client for Solana swaps, pricing, limit orders, and lending.
pub mod jupiter;
/// Kamino Vaults API client and vault/position types.
pub mod kamino;
/// Morpho GraphQL API client and vault/user query types.
pub mod morpho;
/// 0x Swap API v2 client and quote types.
pub mod zerox;
