use anyhow::{Ok, Result};
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

fn init_config(fname: &Path) -> Result<()> {
    let cfg = Config::default();
    let cfg_str = toml::to_string(&cfg)?;

    fs::write(fname, cfg_str)?;

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub api_url: String,
    pub api_key: String,
}

impl Config {
    fn default() -> Self {
        Self {
            api_url: "https://pro-api.coinmarketcap.com".to_string(),
            api_key: "ADD_YOUR_KEY".to_string(),
        }
    }

    pub fn load_account() -> Result<Self> {
        let fname = data_dir().unwrap().join("bonanca/config.toml");

        if !fname.is_file() {
            println!("No config file found, initializing default config file.");
            init_config(&fname)?;
        }

        let buf = fs::read_to_string(fname)?;
        let cfg: Config = toml::from_str(&buf)?;

        Ok(cfg)
    }
}
