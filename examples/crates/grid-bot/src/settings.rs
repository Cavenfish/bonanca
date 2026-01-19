use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GridBotSettings {
    pub chain_id: Option<u16>,
    pub rpc_url: String,
    pub keyvault: PathBuf,
    pub child: u32,
    pub aggregator: ApiInfo,
    pub trading_pair: TradePair,
}

impl GridBotSettings {
    pub fn load(fname: &Path) -> Self {
        let file = File::open(fname).expect("Could not open file");
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).expect("Check JSON file")
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiInfo {
    pub name: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradePair {
    pub token_a: Token,
    pub token_b: Token,
    pub num_grids: u8,
    pub upper_limit: f64,
    pub lower_limit: f64,
    pub buy_amount: f64,
    pub sell_amount: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub address: String,
    pub decimals: i32,
}
