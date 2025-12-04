use anyhow::Result;
use bonanca_api_lib::lending_oracle::get_lending_rates;
use bonanca_core::{
    config::Config,
    get_default_config,
    traits::{Bank, Wallet},
};
use bonanca_lending::{evm::aave::AaveV3, solana::kamino::KaminoVault};
use bonanca_wallets::{get_wallet, get_wallet_view};
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
    pub banks: Vec<String>,
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

    // fn get_bank(&self) -> Result<Box<dyn Bank>> {
    //     let wallet = self.get_wallet_view()?;
    //     let pubkey = wallet.get_pubkey()?;

    //     let (rpc_url, keyvault) = self.get_rpc_and_keyvault();

    //     let bank: Box<dyn Bank> = match self.bank.as_str() {
    //         "Aave" => Box::new(AaveV3::view(&self.chain, &pubkey, &rpc_url)),
    //         "Kamino" => Box::new(KaminoVault::new(&keyvault, self.child)),
    //         _ => panic!(),
    //     };

    //     Ok(bank)
    // }

    pub async fn get_token_pools(&self) -> Result<()> {
        let rates = get_lending_rates(&self.banks, "USDC", 137).await?;

        for rate in rates.iter() {
            println!();
            println!("Protocol: {}", rate.protocol);
            println!("APY: {}", rate.apy);
            println!("Name: {}", rate.vault_name);
            println!();
        }

        Ok(())
    }

    pub async fn get_user_data(&self) -> Result<()> {
        // let bank = self.get_bank()?;

        // bank.get_user_data().await?;

        Ok(())
    }
}
