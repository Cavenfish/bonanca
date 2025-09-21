use anyhow::Result;
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    signature::Keypair,
    signer::keypair::{read_keypair_file, write_keypair_file},
};
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    rpc_url: String,
    keypair_file: PathBuf,
}

impl Config {
    fn default() -> Self {
        let kp = Keypair::new();
        let kfile = data_dir().unwrap().join("bonance/keypair.json");
        write_keypair_file(&kp, &kfile);
        Self {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            keypair_file: kfile,
        }
    }
}

fn init_config(fname: &PathBuf) -> Result<()> {
    let cfg = Config::default();
}

pub fn load_account() -> Result<(Keypair, RpcClient)> {
    let fname = data_dir().unwrap().join("bonance/config.toml");
    let buf = fs::read_to_string(fname)?;
    let cfg: Config = toml::from_str(&buf)?;

    let key_pair = read_keypair_file(cfg.keypair_file).unwrap();
    let rpc = RpcClient::new(cfg.rpc_url);

    Ok((key_pair, rpc))
}
