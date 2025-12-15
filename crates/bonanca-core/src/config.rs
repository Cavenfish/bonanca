use std::{
    default::Default,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use dirs::data_dir;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub keyvault: PathBuf,
    pub database: PathBuf,
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

    pub fn update_keyvault(&self, keyvault: PathBuf) {
        let mut new = self.clone();
        new.keyvault = keyvault;
        new.write();
    }

    pub fn update_database(&self, database: PathBuf) {
        let mut new = self.clone();
        new.database = database;
        new.write();
    }
}

impl Default for Config {
    fn default() -> Self {
        let keyvault = data_dir().unwrap().join("bonanca/keyvault.json");
        let database = data_dir().unwrap().join("bonanca/database.redb");

        // let eth_info = ChainInfo {
        //     name: "Ethereum".to_string(),
        //     rpc_url: "wss://0xrpc.io/eth".to_string(),
        //     wrapped_native: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        //     chain_id: Some(1),
        // };

        Self { keyvault, database }
    }
}
