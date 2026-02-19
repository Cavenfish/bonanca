use std::{
    fmt,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::{Ok, Result};
use bonanca::{
    oracle::{CoinMarketCap, DefiLlama},
    wallets::{EvmWallet, HdWalletView, SolWallet},
};
use serde::{Deserialize, Serialize};

use super::rebal_methods::{make_buyin_trades, make_rebal_trades, make_skim_trades};

#[derive(Debug, Deserialize, Serialize)]
pub struct IndexFund {
    pub name: String,
    pub chain: String,
    pub chain_id: Option<u16>,
    pub rpc_url: String,
    pub keyvault: PathBuf,
    pub child: u32,
    pub max_offset: f64,
    pub aggregator: ApiInfo,
    pub oracle: ApiInfo,
    pub sectors: Vec<Sector>,
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

    async fn get_gas_balance(&self) -> Result<f64> {
        let chain = if self.chain.contains(":") {
            self.chain.split(":").next().unwrap()
        } else {
            &self.chain
        };

        match chain {
            "EVM" => {
                let wallet = EvmWallet::view(&self.keyvault, &self.rpc_url, self.child);
                wallet.balance().await
            }
            "Solana" => {
                let wallet = SolWallet::view(&self.keyvault, &self.rpc_url, self.child);
                wallet.balance().await
            }
            _ => panic!(),
        }
    }

    async fn get_asset_balance(&self, asset: &Asset) -> Result<f64> {
        let chain = if self.chain.contains(":") {
            self.chain.split(":").next().unwrap()
        } else {
            &self.chain
        };

        match chain {
            "EVM" => {
                let wallet = EvmWallet::view(&self.keyvault, &self.rpc_url, self.child);
                wallet.token_balance(&asset.address).await
            }
            "Solana" => {
                let wallet = SolWallet::view(&self.keyvault, &self.rpc_url, self.child);
                wallet.token_balance(&asset.address).await
            }
            _ => panic!(),
        }
    }

    async fn get_asset_value(&self, asset: &Asset, amount: f64, chain: &str) -> Result<f64> {
        match self.oracle.name.as_str() {
            "DefiLlama" => {
                let oracle = DefiLlama::new();
                oracle.get_token_price(&asset.address, amount, chain).await
            }
            "CoinMarketCap" => {
                let oracle = CoinMarketCap::new(self.oracle.api_key.clone());
                oracle.get_token_price(&asset.symbol, amount).await
            }
            _ => panic!(),
        }
    }

    pub async fn get_balances(&self) -> Result<IndexBalances> {
        let chain = if self.chain.contains(":") {
            self.chain.split(":").last().unwrap()
        } else {
            &self.chain
        };

        let gas = self.get_gas_balance().await?;
        let mut total = 0.0;
        let mut balances: Vec<AssetBalance> = Vec::new();

        for sector in &self.sectors {
            let target = sector.weight / (sector.assets.len() as f64);
            for asset in &sector.assets {
                let bal = self.get_asset_balance(asset).await?;

                let usd = if bal != 0.0 {
                    self.get_asset_value(asset, bal, chain).await?
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

        if self.auxiliary_assets.is_some() {
            for asset in self.auxiliary_assets.as_ref().unwrap() {
                let bal = self.get_asset_balance(asset).await?;

                let usd = if bal != 0.0 {
                    self.get_asset_value(asset, bal, chain).await?
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
            "redistribute" => make_rebal_trades(bals, self.max_offset)?,
            "sell" => make_skim_trades(bals, aux_token, self.max_offset)?,
            "buy" => {
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
    pub api_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sector {
    pub name: String,
    pub assets: Vec<Asset>,
    pub weight: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Asset {
    pub name: String,
    pub symbol: String,
    pub address: String,
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
    pub from_name: String,
    pub to: String,
    pub to_name: String,
    pub amount: f64,
}

impl fmt::Display for RebalTrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Trade {} {} for {}",
            self.amount, self.from_name, self.to_name
        )
    }
}
