use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Deserialize, Serialize)]
pub struct IndexFund {
    pub name: String,

    pub sectors: Vec<Sector>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sector {
    pub name: String,

    pub assets: Vec<Asset>,

    pub weight: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Asset {
    pub name: String,

    pub token: Pubkey,
}
