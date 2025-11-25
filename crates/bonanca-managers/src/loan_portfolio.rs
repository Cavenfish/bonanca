use anyhow::Result;
use bonanca_core::{
    get_wallet, get_wallet_view,
    wallets::{evm::EvmWallet, traits::Wallet},
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
    pub chain_id: Option<u16>,
    pub evm_chain: Option<String>,
    pub child: u32,
    pub rpc_url: String,
    pub keyvault: PathBuf,
    pub gas_address: String,
}

impl LoanPortfolio {
    pub fn load(fname: &Path) -> Self {
        let file = File::open(fname).expect("Could not open file");
        let reader = BufReader::new(file);
        let port: LoanPortfolio = serde_json::from_reader(reader).expect("Check JSON file");

        port
    }

    pub fn get_wallet(&self) -> Result<Box<dyn Wallet + Send + Sync>> {
        get_wallet(&self.chain, &self.keyvault, &self.rpc_url, self.child)
    }

    pub fn get_wallet_view(&self) -> Result<Box<dyn Wallet + Send + Sync>> {
        get_wallet_view(&self.chain, &self.keyvault, &self.rpc_url, self.child)
    }

    pub async fn get_pools(&self) -> Result<()> {
        let wallet = EvmWallet::view(&self.keyvault, &self.rpc_url, self.child);

        let client = wallet.get_view_client();

        let chain = if &self.chain == "EVM" {
            self.evm_chain.as_ref().unwrap()
        } else {
            &self.chain
        };

        let aave = AaveV3::new(chain, &wallet.pubkey.to_string(), client);

        let _ = aave.get_pools().await?;

        Ok(())
    }
}
