use anyhow::Result;

use crate::defi::{aave::AaveV3Api, kamino::KaminoApi, morpho::MorphoApi};

pub struct LendingRate {
    pub apy: f64,
    pub protocol: String,
    pub token: String,
    pub vault_name: String,
}
