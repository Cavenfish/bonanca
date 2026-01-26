use anyhow::Result;
use bonanca_api_lib::price_feeds::{
    cmc::CoinMarketCapApi,
    defi_llama::DefiLlamaApi,
    dexscreener::{DexScreenerApi, DexScreenerPairData},
};

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

pub struct DexScreener {
    api: DexScreenerApi,
}

impl DexScreener {
    pub fn new() -> Self {
        let api = DexScreenerApi::new();
        Self { api }
    }

    pub async fn get_pair_data(&self, chain: &str, pair: &str) -> Result<DexScreenerPairData> {
        let resp = self.api.get_pair_data(chain, pair).await?;

        match resp.pair {
            Some(pair) => Ok(pair),
            None => Err(anyhow::anyhow!("Pair not found")),
        }
    }

    pub async fn get_pairs_from_query(&self, query: &str) -> Result<Vec<DexScreenerPairData>> {
        let resp = self.api.get_pairs_from_query(query).await?;

        match resp.pairs {
            Some(pairs) => Ok(pairs),
            None => Err(anyhow::anyhow!("Pair not found")),
        }
    }

    pub async fn get_token_pairs(
        &self,
        chain: &str,
        token: &str,
    ) -> Result<Vec<DexScreenerPairData>> {
        self.api.get_token_pairs(chain, token).await
    }
}
