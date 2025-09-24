use anyhow::{Ok, Result};
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    signature::Keypair,
    signer::keypair::{read_keypair_file, write_keypair_file},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn init_config(fname: &Path) -> Result<()> {
    let cfg = Config::default();
    let cfg_str = toml::to_string(&cfg)?;

    fs::write(fname, cfg_str)?;

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub rpc_url: String,
    pub api_url: String,
    pub api_key: String,
    pub keypair_file: PathBuf,
}

impl Config {
    fn default() -> Self {
        let kp = Keypair::new();
        let kfile = data_dir().unwrap().join("bonance/keypair.json");
        write_keypair_file(&kp, &kfile).unwrap();
        Self {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            api_url: "https://pro-api.coinmarketcap.com".to_string(),
            api_key: "ADD_YOUR_KEY".to_string(),
            keypair_file: kfile,
        }
    }

    pub fn load_account() -> Result<Self> {
        let fname = data_dir().unwrap().join("bonance/config.toml");

        if !fname.is_file() {
            println!("No config file found, initializing default config file.");
            init_config(&fname)?;
        }

        let buf = fs::read_to_string(fname)?;
        let cfg: Config = toml::from_str(&buf)?;

        Ok(cfg)
    }

    pub fn get_rpc_client(&self) -> Result<RpcClient> {
        let rpc = RpcClient::new(self.rpc_url.clone());

        Ok(rpc)
    }

    pub fn get_keypair(&self) -> Result<Keypair> {
        let kp = read_keypair_file(self.keypair_file.clone()).unwrap();

        Ok(kp)
    }
}
