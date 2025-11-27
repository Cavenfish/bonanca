use std::{
    default::Default,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use dirs::data_dir;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
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

    pub fn write(&self) {
        let fname = data_dir().unwrap().join("bonanca/config.json");
        let file = File::create(fname).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self).unwrap();
    }

    pub fn add_chain_info(&self, chain_info: ChainInfo) {
        let mut new = self.clone();
        new.chains_info.push(chain_info);
        new.write();
    }

    pub fn update_chain_info(&self, chain_info: ChainInfo) {
        let mut new = self.clone();

        if let Some(pos) = new
            .chains_info
            .iter()
            .position(|c| c.name == chain_info.name)
        {
            new.chains_info.remove(pos);
        } else {
            println!("No existing chain with name \"{}\" found", chain_info.name);
            println!("\nCheck for typos, or consider adding the new chain");
            return;
        }

        new.chains_info.push(chain_info);
        new.write();
    }

    pub fn update_keyvault(&self, keyvault: PathBuf) {
        let mut new = self.clone();
        new.keyvault = keyvault;
        new.write();
    }

    pub fn get_default_chain_rpc(&self, chain: &str) -> String {
        let config = Config::load();
        let name = if chain.contains(":") {
            chain.split(":").last().unwrap()
        } else {
            chain
        };

        config
            .chains_info
            .iter()
            .find(|c| c.name == name)
            .unwrap()
            .rpc_url
            .clone()
    }

    pub fn get_default_chain_id(&self, chain: &str) -> Option<u16> {
        let config = Config::load();
        let name = if chain.contains(":") {
            chain.split(":").last().unwrap()
        } else {
            chain
        };

        config
            .chains_info
            .iter()
            .find(|c| c.name == name)
            .unwrap()
            .chain_id
    }

    pub fn get_default_wrapped_native(&self, chain: &str) -> String {
        let config = Config::load();
        let name = if chain.contains(":") {
            chain.split(":").last().unwrap()
        } else {
            chain
        };

        config
            .chains_info
            .iter()
            .find(|c| c.name == name)
            .unwrap()
            .wrapped_native
            .clone()
    }
}

impl Default for Config {
    fn default() -> Self {
        let keyvault = data_dir().unwrap().join("bonanca/keyvault.json");

        let eth_info = ChainInfo {
            name: "Ethereum".to_string(),
            rpc_url: "wss://0xrpc.io/eth".to_string(),
            wrapped_native: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            chain_id: Some(1),
        };

        Self {
            keyvault,
            chains_info: vec![eth_info],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChainInfo {
    pub name: String,
    pub rpc_url: String,
    pub wrapped_native: String,
    pub chain_id: Option<u16>,
}
