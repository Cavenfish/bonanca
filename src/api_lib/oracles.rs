use anyhow::Result;

pub trait Oracle {
    async fn get_token_value(&self, token: &str, amount: f64) -> Result<f64>;
}
