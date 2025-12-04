use anyhow::Result;

use crate::defi::{aave::AaveApi, morpho::MorphoApi};

pub struct LendingRate {
    pub apy: f64,
    pub protocol: String,
    pub token: String,
    pub vault_name: String,
}

pub async fn get_lending_rates(
    banks: &Vec<String>,
    token: &str,
    chain_id: u64,
) -> Result<Vec<LendingRate>> {
    let mut rates: Vec<LendingRate> = Vec::new();

    for bank in banks.iter() {
        match bank.as_str() {
            "Aave" => {
                let api = AaveApi::new();
                let mut tmp = api.query_market_v3(token, chain_id).await?;
                rates.append(&mut tmp);
            }
            "Morpho" => {
                let api = MorphoApi::new();
                let mut tmp = api.query_vaults_v1(token, chain_id).await?;
                rates.append(&mut tmp);
            }
            _ => panic!(),
        }
    }

    Ok(rates)
}
