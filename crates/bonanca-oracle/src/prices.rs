use anyhow::Result;
use bonanca_api_lib::price_feeds::{cmc::CoinMarketCapApi, defi_llama::DefiLlamaApi};

pub struct CoinMarketCap {
    api: CoinMarketCapApi,
}

impl CoinMarketCap {
    pub fn new(api_key: String) -> Self {
        let api = CoinMarketCapApi::new(api_key);
        Self { api }
    }

    pub async fn get_token_price(&self, symbol: &str, amount: f64) -> Result<f64> {
        let quote = self.api.get_price_quote(symbol, amount).await?;

        let data = quote.data.first().unwrap();

        let value = data.quote.usd.price.unwrap();

        Ok(value)
    }
}

pub struct DefiLlama {
    api: DefiLlamaApi,
}

impl DefiLlama {
    pub fn new() -> Self {
        let api = DefiLlamaApi::new();
        Self { api }
    }

    pub async fn get_token_price(&self, token: &str, amount: f64, chain: &str) -> Result<f64> {
        let quote = self.api.get_price_quote(chain, token).await?;
        let price = quote.coins.values().next().unwrap().price;
        let value = price * amount;

        Ok(value)
    }
}
