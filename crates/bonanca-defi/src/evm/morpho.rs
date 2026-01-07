use alloy::{
    primitives::{Address, U256, address},
    providers::{DynProvider, Provider, ProviderBuilder},
    signers::{k256::ecdsa::SigningKey, local::LocalSigner},
    sol,
    transports::http::reqwest::Url,
};
use anyhow::Result;
use std::str::FromStr;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    PoolV3,
    "src/evm/ABI/morpho_vault_v1.json"
}

pub struct MorphoVaultV1 {
    pub user: Address,
    pub vault: Address,
    pub client: DynProvider,
}

impl MorphoVaultV1 {
    pub fn new(vault_str: &str, rpc_url: &str, signer: LocalSigner<SigningKey>) -> Self {
        let user = signer.address();
        let vault = Address::from_str(vault_str).unwrap();

        let rpc = Url::from_str(rpc_url).unwrap();
        let client: DynProvider = ProviderBuilder::new()
            .wallet(signer)
            .connect_http(rpc)
            .erased();

        Self {
            user,
            vault,
            client,
        }
    }

    async fn get_user_data(&self) -> Result<()> {
        Ok(())
    }

    async fn get_token_pools(&self, token: &str) -> Result<()> {
        Ok(())
    }

    async fn supply(&self, token: &str, amount: u64) -> Result<()> {
        Ok(())
    }

    async fn borrow(&self, token: &str, amount: u64) -> Result<()> {
        Ok(())
    }

    async fn repay(&self, token: &str, amount: u64) -> Result<()> {
        Ok(())
    }

    async fn withdraw(&self, token: &str, amount: u64) -> Result<()> {
        Ok(())
    }
}
