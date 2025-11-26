use std::{
    default::Default,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use dirs::data_dir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub keyvault: PathBuf,
    pub chains_info: Vec<ChainInfo>,
}

impl Config {
    pub fn load() -> Self {
        let fname = data_dir().unwrap().join("bonanca/config.json");
        let file = File::open(fname).unwrap();
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader).unwrap();

        config
    }

    pub fn write(&self, filename: &Path) {
        let file = File::create(filename).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self).unwrap();
    }
}

impl Default for Config {
    fn default() -> Self {
        let keyvault = data_dir().unwrap().join("bonanca/keyvault.json");

        let eth_info = ChainInfo {
            name: "Ethereum".to_string(),
            rpc_url: "wss://0xrpc.io/eth".to_string(),
            gas_token: "0x0000000000000000000000000000000000000000".to_string(),
            chain_id: Some(1),
        };

        Self {
            keyvault,
            chains_info: vec![eth_info],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChainInfo {
    pub name: String,
    pub rpc_url: String,
    pub gas_token: String,
    pub chain_id: Option<u16>,
}
