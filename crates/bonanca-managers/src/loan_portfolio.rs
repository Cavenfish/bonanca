use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::Result;
use bonanca_api_lib::lending_oracle::get_lending_rates;
use bonanca_core::{
    config::Config,
    get_default_config,
    traits::{Bank, Wallet},
};
use bonanca_db::BonancaDB;
use bonanca_lending::{evm::aave::AaveV3, solana::kamino::KaminoVault};
use bonanca_wallets::{get_wallet, get_wallet_view};
use serde::{Deserialize, Serialize};

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

    fn get_keyvault(&self) -> Result<&PathBuf> {
        let keyvault = match &self.keyvault {
            Some(file) => file,
            None => &self.config.keyvault,
        };

        Ok(keyvault)
    }

    fn get_rpc_url(&self) -> Result<String> {
        match &self.rpc_url {
            Some(url) => Ok(url.clone()),
            None => {
                let db = BonancaDB::load();
                let info = db.read_chain_info(&self.chain)?;
                Ok(info.rpc_url)
            }
        }
    }

    fn get_chain_id(&self) -> Result<Option<u16>> {
        let db = BonancaDB::load();
        let info = db.read_chain_info(&self.chain)?;
        Ok(info.chain_id)
    }

    fn get_api_key(&self, name: &str, maybe_key: Option<String>) -> Result<String> {
        match maybe_key {
            Some(api_key) => Ok(api_key),
            None => {
                let db = BonancaDB::load();
                db.get_api_key(name)
            }
        }
    }

    pub fn get_wallet(&self) -> Result<Box<dyn Wallet + Send + Sync>> {
        let keyvault = self.get_keyvault()?;
        let rpc_url = self.get_rpc_url()?;
        get_wallet(&self.chain, &keyvault, &rpc_url, self.child)
    }

    pub fn get_wallet_view(&self) -> Result<Box<dyn Wallet + Send + Sync>> {
        let keyvault = self.get_keyvault()?;
        let rpc_url = self.get_rpc_url()?;
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

        let best = rates.iter().max_by(|a, b| a.apy.total_cmp(&b.apy)).unwrap();

        println!("Best Rate:");
        println!("Protocol: {}", best.protocol);
        println!("APY: {}", best.apy);
        println!("Name: {}", best.vault_name);

        Ok(())
    }

    pub async fn get_user_data(&self) -> Result<()> {
        // let bank = self.get_bank()?;

        // bank.get_user_data().await?;

        Ok(())
    }
}
