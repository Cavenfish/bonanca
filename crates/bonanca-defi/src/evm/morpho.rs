use alloy::{primitives::Address, providers::DynProvider, sol};
use anyhow::Result;
use bonanca_api_lib::defi::morpho::MorphoApi;
use std::str::FromStr;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    PoolV3,
    "src/evm/ABI/morpho_vault_v1.json"
}

pub struct MorphoVaultV1 {
    api: MorphoApi,
}

impl MorphoVaultV1 {
    pub fn new() -> Self {
        let api = MorphoApi::new();

        Self { api }
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
