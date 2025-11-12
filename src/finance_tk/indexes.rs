use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::{
    api_lib::{
        cmc::CoinMarketCap,
        jupiter::Jupiter,
        traits::{AllExchanges, Exchange, Oracle},
        zerox::ZeroX,
    },
    wallets::{evm::EvmWallet, solana::SolWallet, traits::Wallet},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct IndexFund {
    pub name: String,
    pub chain: String,
    pub chain_id: Option<u16>,
    pub max_offset: f64,
    pub rpc_url: String,
    pub keystore: PathBuf,
    pub aggregator: ApiInfo,
    pub oracle: ApiInfo,
    pub sectors: Vec<Sector>,
}

impl IndexFund {
    pub fn load(fname: &Path) -> Self {
        let file = File::open(fname).expect("Could not open file");
        let reader = BufReader::new(file);
        let fund: IndexFund = serde_json::from_reader(reader).expect("Check JSON file");

        fund
    }

    pub fn get_wallet(&self) -> Result<Box<dyn Wallet + Send + Sync>> {
        let wallet: Box<dyn Wallet + Send + Sync> = match self.chain.as_str() {
            "EVM" => Box::new(EvmWallet::load(&self.keystore, &self.rpc_url)),
            "Solana" => Box::new(SolWallet::load(&self.keystore, &self.rpc_url)),
            _ => Err(anyhow::anyhow!("Unsupported chain"))?,
        };

        Ok(wallet)
    }

    pub fn get_oracle(&self) -> Result<Box<dyn Oracle>> {
        let oracle: Box<dyn Oracle> = match self.oracle.name.as_str() {
            "CoinMarketCap" => Box::new(CoinMarketCap::new(
                self.oracle.api_url.clone(),
                self.oracle.api_key.clone(),
            )),
            "Jupiter" => Box::new(Jupiter::new(
                self.oracle.api_url.clone(),
                self.oracle.api_key.clone(),
            )),
            _ => Err(anyhow::anyhow!("Unsupported oracle"))?,
        };

        Ok(oracle)
    }

    pub fn get_exchange(&self) -> Result<Box<dyn Exchange>> {
        let exchange: Box<dyn Exchange> = match self.aggregator.name.as_str() {
            "0x" => Box::new(ZeroX::new(
                self.aggregator.api_url.clone(),
                self.aggregator.api_key.clone(),
                self.chain_id.unwrap(),
            )),
            "Jupiter" => Box::new(Jupiter::new(
                self.aggregator.api_url.clone(),
                self.aggregator.api_key.clone(),
            )),
            _ => Err(anyhow::anyhow!("Unsupported aggregator"))?,
        };

        Ok(exchange)
    }

    pub async fn get_balances(&self) -> Result<IndexBalances> {
        let wallet = self.get_wallet()?;
        let oracle = self.get_oracle()?;

        let mut total = 0.0;
        let mut balances: Vec<AssetBalance> = Vec::new();

        for sector in &self.sectors {
            let target = sector.weight / (sector.assets.len() as f64);
            for asset in &sector.assets {
                let bal = wallet.token_balance(&asset.address).await?;

                let usd = if bal != 0.0 {
                    oracle.get_token_value(&asset, bal).await?
                } else {
                    0.0
                };

                balances.push(AssetBalance {
                    name: asset.name.clone(),
                    addy: asset.address.clone(),
                    amount: bal,
                    value: usd,
                    target: target,
                });

                total += usd;
            }
        }

        Ok(IndexBalances {
            total: total,
            balances: balances,
        })
    }

    pub fn get_trades(&self, bals: &IndexBalances) -> Result<Vec<RebalTrade>> {
        let mut addys: Vec<String> = Vec::new();
        let mut diffs: Vec<f64> = Vec::new();
        let mut amounts: Vec<f64> = Vec::new();
        let mut actuals: Vec<f64> = Vec::new();

        for asset in &bals.balances {
            let bal = asset.value;
            let actual = bal / bals.total;
            let diff = asset.target - actual;

            addys.push(asset.addy.clone());
            diffs.push(diff);
            amounts.push(asset.amount);
            actuals.push(actual);
        }

        let n = diffs.len();

        let mut order = (0..n).collect::<Vec<_>>();
        order.sort_by_key(|&k| (&diffs[k] * 1e6) as i64);

        let mut trades: Vec<RebalTrade> = Vec::new();

        for i in 0..(n - 1) {
            let small = order[i];

            let mut j = n - 1;
            while diffs[small].abs() > self.max_offset {
                let big = order[j];

                if diffs[big] < 0.0 {
                    println!("Two negative numbers");
                    break;
                }

                let diff = if diffs[big].abs() > diffs[small].abs() {
                    diffs[small].abs()
                } else {
                    diffs[big].abs()
                };

                if diff == 0.0 {
                    j -= 1;
                    continue;
                }

                let frac = diff / actuals[small];
                let amount = frac * amounts[small];

                trades.push(RebalTrade {
                    from: addys[small].clone(),
                    to: addys[big].clone(),
                    amount: amount,
                });

                diffs[small] += diff;
                diffs[big] -= diff;
                j -= 1;
            }
        }

        Ok(trades)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiInfo {
    pub name: String,
    pub api_url: String,
    pub api_key: String,
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
    pub symbol: String,
    pub address: String,
}

pub struct IndexBalances {
    pub total: f64,
    pub balances: Vec<AssetBalance>,
}

pub struct AssetBalance {
    pub name: String,
    pub addy: String,
    pub amount: f64,
    pub value: f64,
    pub target: f64,
}

#[derive(Debug)]
pub struct RebalTrade {
    pub from: String,
    pub to: String,
    pub amount: f64,
}
