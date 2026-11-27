//! HTTP and GraphQL API clients used across the Bonanca workspace.
//!
//! This crate wraps external REST and GraphQL endpoints for:
//!
//! - [`block_explorer`]: On-chain transaction history lookups (Etherscan V2).
//! - [`defi`]: Decentralized exchange aggregators and lending protocols (Aave V3, `CoW` Protocol,
//!   Jupiter, Kamino, Morpho, and 0x).
//! - [`price_feeds`]: Token price and market data providers (`CoinMarketCap`, `CoinGecko`,
//!   `DefiLlama`, and `DexScreener`).

/// Block explorer API clients.
pub mod block_explorer;
/// DeFi protocol API clients.
pub mod defi;
/// Price feed and market data API clients.
pub mod price_feeds;
