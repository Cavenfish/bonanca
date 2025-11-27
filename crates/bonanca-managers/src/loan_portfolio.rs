use anyhow::Result;
use bonanca_core::{
    config::Config,
    get_default_config, get_wallet, get_wallet_view,
    wallets::{self, evm::EvmWallet, traits::Wallet},
};
use bonanca_lending::evm::aave::AaveV3;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct LoanPortfolio {
    pub name: String,
    pub chain: String,
    pub child: u32,
    pub rpc_url: Option<String>,
    pub keyvault: Option<PathBuf>,

    #[serde(default = "get_default_config")]
    pub config: Config,
}

impl LoanPortfolio {
    pub fn load(fname: &Path) -> Self {
        let file = File::open(fname).expect("Could not open file");
        let reader = BufReader::new(file);
        let port: LoanPortfolio = serde_json::from_reader(reader).expect("Check JSON file");

        port
    }

    fn get_rpc_and_keyvault(&self) -> (String, PathBuf) {
        let rpc_url = if self.rpc_url.is_none() {
            self.config.get_default_chain_rpc(&self.chain)
        } else {
            self.rpc_url.clone().unwrap()
        };

        let keyvault = if self.keyvault.is_none() {
            self.config.keyvault.clone()
        } else {
            self.keyvault.clone().unwrap()
        };

        (rpc_url, keyvault)
    }

    pub fn get_wallet(&self) -> Result<Box<dyn Wallet + Send + Sync>> {
        let (rpc_url, keyvault) = self.get_rpc_and_keyvault();
        get_wallet(&self.chain, &keyvault, &rpc_url, self.child)
    }

    pub fn get_wallet_view(&self) -> Result<Box<dyn Wallet + Send + Sync>> {
        let (rpc_url, keyvault) = self.get_rpc_and_keyvault();
        get_wallet_view(&self.chain, &keyvault, &rpc_url, self.child)
    }

    pub async fn get_user_data(&self) -> Result<()> {
        let wallet = self.get_wallet_view()?;
        let pubkey = wallet.get_pubkey()?;

        let rpc_url = if self.rpc_url.is_none() {
            self.config.get_default_chain_rpc(&self.chain)
        } else {
            self.rpc_url.clone().unwrap()
        };

        let aave = AaveV3::view(&self.chain, &pubkey, &rpc_url);

        let _ = aave.get_user_data().await?;

        Ok(())
    }
}
