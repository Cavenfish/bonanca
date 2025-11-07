use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::{
    api_lib::{
        cmc::CoinMarketCap,
        jupiter::Jupiter,
        traits::{Exchange, Oracle},
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

    pub fn get_wallet(&self) -> Result<Box<dyn Wallet>> {
        let wallet: Box<dyn Wallet> = match self.chain.as_str() {
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

    pub async fn get_balances(&self) -> Result<IndexBalances> {
        let wallet = self.get_wallet()?;
        let oracle = self.get_oracle()?;

        let mut total = 0.0;
        let mut balances: HashMap<String, f64> = HashMap::new();

        for sector in &self.sectors {
            for asset in &sector.assets {
                let bal = wallet.token_balance(&asset.address).await?;

                let usd = if bal != 0.0 {
                    oracle.get_token_value(&asset, bal).await?
                } else {
                    0.0
                };

                balances.insert(asset.name.clone(), usd);

                total += usd;
            }
        }

        Ok(IndexBalances {
            total: total,
            balances: balances,
        })
    }

    pub fn get_trades(&self, bals: &IndexBalances) -> Result<HashMap<String, f64>> {
        let mut trades: HashMap<String, f64> = HashMap::new();

        for sector in &self.sectors {
            let target = sector.weight / (sector.assets.len() as f64);
            for asset in &sector.assets {
                let bal = bals.balances.get(&asset.name).unwrap();
                let actual = bal / bals.total;
                let trade = (target - actual) * bals.total;

                trades.insert(asset.name.clone(), trade);
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
    pub balances: HashMap<String, f64>,
}
