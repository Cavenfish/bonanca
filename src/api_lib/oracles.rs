use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Oracle {
    async fn get_token_value(&self, token: &str, amount: f64) -> Result<f64>;
}
