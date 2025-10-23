use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct IndexFund {
    pub name: String,
    pub chain: String,
    pub chain_id: Option<u16>,
    pub rpc_url: String,
    pub keystore: PathBuf,
    pub aggregator: ApiInfo,
    pub oracle: ApiInfo,
    pub sectors: Vec<Sector>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiInfo {
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

    pub token: String,
}

pub fn load_index_fund(fname: &Path) -> Result<IndexFund> {
    let file = File::open(fname)?;
    let reader = BufReader::new(file);
    let fund: IndexFund = serde_json::from_reader(reader)?;

    Ok(fund)
}
