use anyhow::{Ok, Result};
use bonanca_core::{
    api_lib::traits::{Exchange, Oracle},
    get_exchange, get_oracle, get_wallet, get_wallet_view,
    holdings::Asset,
    wallets::traits::Wallet,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::rebal_methods::{make_buyin_trades, make_rebal_trades, make_skim_trades};

#[derive(Debug, Deserialize, Serialize)]
pub struct IndexFund {
    pub name: String,
    pub chain: String,
    pub chain_id: Option<u16>,
    pub evm_chain: Option<String>,
    pub child: u32,
    pub max_offset: f64,
    pub rpc_url: String,
    pub keyvault: PathBuf,
    pub aggregator: ApiInfo,
    pub oracle: ApiInfo,
    pub sectors: Vec<Sector>,
    pub gas_address: String,
    pub auxiliary_assets: Option<Vec<Asset>>,
}

impl IndexFund {
    pub fn load(fname: &Path) -> Self {
        let file = File::open(fname).expect("Could not open file");
        let reader = BufReader::new(file);
        let fund: IndexFund = serde_json::from_reader(reader).expect("Check JSON file");

        let weights: Vec<f64> = fund.sectors.iter().map(|s| s.weight).collect();

        assert_eq!(weights.iter().sum::<f64>(), 1.0);

        fund
    }

    pub fn get_wallet(&self) -> Result<Box<dyn Wallet + Send + Sync>> {
        get_wallet(&self.chain, &self.keyvault, &self.rpc_url, self.child)
    }

    pub fn get_wallet_view(&self) -> Result<Box<dyn Wallet + Send + Sync>> {
        get_wallet_view(&self.chain, &self.keyvault, &self.rpc_url, self.child)
    }

    pub fn get_oracle(&self) -> Result<Box<dyn Oracle>> {
        get_oracle(
            &self.oracle.name,
            &self.oracle.api_url,
            &self.oracle.api_key,
        )
    }

    pub fn get_exchange(&self) -> Result<Box<dyn Exchange>> {
        get_exchange(
            &self.aggregator.name,
            &self.aggregator.api_url,
            &self.aggregator.api_key,
            self.chain_id,
        )
    }

    pub fn get_all_assets(&self) -> Result<Vec<Asset>> {
        let mut assets: Vec<Asset> = Vec::new();

        self.sectors
            .iter()
            .for_each(|s| s.assets.iter().for_each(|a| assets.push(a.clone())));

        if let Some(aux_assets) = &self.auxiliary_assets {
            assets.extend(aux_assets.clone())
        };

        Ok(assets)
    }

    pub async fn get_balances(&self) -> Result<IndexBalances> {
        let wallet = self.get_wallet_view()?;
        let oracle = self.get_oracle()?;
        let chain = if &self.chain == "EVM" {
            self.evm_chain.as_ref().unwrap()
        } else {
            &self.chain
        };

        let gas = wallet.balance().await?;
        let mut total = 0.0;
        let mut balances: Vec<AssetBalance> = Vec::new();

        for sector in &self.sectors {
            let target = sector.weight / (sector.assets.len() as f64);
            for asset in &sector.assets {
                let bal = wallet.token_balance(&asset.address).await?;

                let usd = if bal != 0.0 {
                    oracle.get_token_value(asset, bal, &chain).await?
                } else {
                    0.0
                };

                balances.push(AssetBalance {
                    name: asset.name.clone(),
                    addy: asset.address.clone(),
                    amount: bal,
                    value: usd,
                    target,
                });

                total += usd;
            }
        }

        let mut aux_balances: Vec<AuxAssetBalance> = Vec::new();

        for asset in self.auxiliary_assets.as_ref().unwrap() {
            let bal = wallet.token_balance(&asset.address).await?;

            let usd = if bal != 0.0 {
                oracle.get_token_value(asset, bal, &chain).await?
            } else {
                0.0
            };

            aux_balances.push(AuxAssetBalance {
                name: asset.name.clone(),
                addy: asset.address.clone(),
                amount: bal,
                value: usd,
            });
        }

        Ok(IndexBalances {
            gas,
            total,
            balances,
            aux_balances,
        })
    }

    pub fn get_trades(
        &self,
        bals: &IndexBalances,
        method: &str,
        aux_token: &str,
    ) -> Result<Vec<RebalTrade>> {
        let trades = match method {
            "rebalance" => make_rebal_trades(&bals, self.max_offset)?,
            "skim" => make_skim_trades(bals, aux_token, self.max_offset)?,
            "buyin" => {
                let from_asset = bals
                    .aux_balances
                    .iter()
                    .find(|x| x.addy == aux_token)
                    .unwrap();
                let usd_per_from_token = from_asset.value / from_asset.amount;
                make_buyin_trades(bals, aux_token, usd_per_from_token, self.max_offset)?
            }
            _ => panic!(),
        };

        Ok(trades)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiInfo {
    pub name: String,
    pub api_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sector {
    pub name: String,
    pub assets: Vec<Asset>,
    pub weight: f64,
}

pub struct IndexBalances {
    pub gas: f64,
    pub total: f64,
    pub balances: Vec<AssetBalance>,
    pub aux_balances: Vec<AuxAssetBalance>,
}

pub struct AssetBalance {
    pub name: String,
    pub addy: String,
    pub amount: f64,
    pub value: f64,
    pub target: f64,
}

pub struct AuxAssetBalance {
    pub name: String,
    pub addy: String,
    pub amount: f64,
    pub value: f64,
}

#[derive(Debug)]
pub struct RebalTrade {
    pub from: String,
    pub to: String,
    pub amount: f64,
}

impl fmt::Display for RebalTrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Trade {} {} for {}", self.amount, self.from, self.to)
    }
}
