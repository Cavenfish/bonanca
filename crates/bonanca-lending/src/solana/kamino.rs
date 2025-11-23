use anyhow::Result;
use async_trait::async_trait;

use crate::traits::Lender;

pub struct KaminoVault {
    pub user: String,
}

#[async_trait]
impl Lender for KaminoVault {
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
