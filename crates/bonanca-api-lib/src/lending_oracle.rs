//! Lending rate aggregation across protocols.

use anyhow::Result;

use crate::defi::{aave::AaveV3Api, kamino::KaminoApi, morpho::MorphoApi};

/// Lending rate for a token in a protocol vault.
pub struct LendingRate {
    /// Annual percentage yield as a decimal fraction.
    pub apy: f64,
    /// Protocol name (e.g. `"Aave"`, `"Kamino"`, `"Morpho"`).
    pub protocol: String,
    /// Token symbol.
    pub token: String,
    /// Vault name.
    pub vault_name: String,
}
