use anyhow::Result;
use async_trait::async_trait;
use bonanca_core::api_lib::kamino::KaminoApi;

use crate::traits::Bank;

pub struct KaminoVault {
    pub user: String,
}

#[async_trait]
impl Bank for KaminoVault {
    async fn get_pools(&self) -> Result<()> {
        let kamino_api = KaminoApi::new();

        let pools = kamino_api.get_all_kvaults().await?;

        for pool in pools.iter() {
            println!("Pool Address: {}", pool.address);
            println!("Token Available: {}", pool.state.token_available);
            println!("Min Deposit Amount: {}", pool.state.min_deposit_amount);
            println!("Performance Fee: {} bps", pool.state.performance_fee_bps);
            println!("Management Fee: {} bps", pool.state.management_fee_bps);
        }

        Ok(())
    }

    async fn get_user_data(&self) -> Result<()> {
        let kamino_api = KaminoApi::new();

        let data = kamino_api.get_user_data(&self.user).await?;

        data.iter().for_each(|f| println!("{}", f));

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

impl KaminoVault {
    pub fn view(user: &str) -> Self {
        Self {
            user: user.to_string(),
        }
    }
}
