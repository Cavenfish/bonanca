use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Bank {
    async fn get_pools(&self) -> Result<()>;

    async fn get_user_data(&self) -> Result<()>;

    async fn supply(&self, token: &str, amount: u64) -> Result<()>;

    async fn borrow(&self, token: &str, amount: u64) -> Result<()>;

    async fn repay(&self, token: &str, amount: u64) -> Result<()>;

    async fn withdraw(&self, token: &str, amount: u64) -> Result<()>;
}
