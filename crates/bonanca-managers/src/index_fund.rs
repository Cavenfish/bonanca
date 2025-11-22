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

        Ok(IndexBalances {
            gas,
            total,
            balances,
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
                    amount,
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

impl fmt::Display for RebalTrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Trade {} {} for {}", self.amount, self.from, self.to)
    }
}
