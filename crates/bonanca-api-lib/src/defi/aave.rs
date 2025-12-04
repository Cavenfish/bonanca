use alloy_primitives::{Address, ChainId, address};
use anyhow::Result;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;

use crate::lending_oracle::LendingRate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/aave/schema.graphql",
    query_path = "schemas/aave/query.graphql",
    response_derives = "Debug, Clone"
)]
pub struct MarketQuery;

pub struct AaveApi {
    pub base_url: String,
}

impl AaveApi {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.v3.aave.com/graphql".to_string(),
        }
    }

    pub async fn query_market_v3(&self, token: &str, chain_id: u64) -> Result<Vec<LendingRate>> {
        let client = Client::new();
        let address = match chain_id {
            // Ethereum
            1 => address!("0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"),
            // Optimism
            10 => address!("0x794a61358D6845594F94dc1DB02A252b5b4814aD"),
            // Gnosis
            100 => address!("0xb50201558B00496A145fE76f7424749556E326D8"),
            // Polygon
            137 => address!("0x794a61358D6845594F94dc1DB02A252b5b4814aD"),
            // Sonic
            146 => address!("0x5362dBb1e601abF3a4c14c22ffEdA64042E5eAA3"),
            // zkSync
            324 => address!("0x78e30497a3c7527d953c6B1E3541b021A98Ac43c"),
            // Metis
            1088 => address!("0x90df02551bB792286e8D4f13E0e357b4Bf1D6a57"),
            // Soneium
            1868 => address!("0xDd3d7A7d03D9fD9ef45f3E587287922eF65CA38B"),
            // Base
            8453 => address!("0xA238Dd80C259a72e81d7e4664a9801593F98d1c5"),
            // Plasma
            9745 => address!("0x925a2A7214Ed92428B5b1B090F80b25700095e12"),
            // Arbitrum
            42161 => address!("0x794a61358D6845594F94dc1DB02A252b5b4814aD"),
            // Celo
            42220 => address!("0x3E59A31363E2ad014dcbc521c4a0d5757d9f3402"),
            // Avalanche
            43114 => address!("0x794a61358D6845594F94dc1DB02A252b5b4814aD"),
            // Ink
            57073 => address!("0x2816cf15F6d2A220E789aA011D5EE4eB6c47FEbA"),
            // Linea
            59144 => address!("0xc47b8C00b0f69a36fa203Ffeac0334874574a8Ac"),
            // Scroll
            534352 => address!("0x11fCfe756c05AD438e312a7fd934381537D3cFfe"),
            _ => Err(anyhow::anyhow!("Unsupported chain ID"))?,
        };

        let market_vars = market_query::MarketRequest {
            address,
            chain_id,
            user: None,
        };

        let variables = market_query::Variables {
            request: market_vars,
        };

        let body = MarketQuery::build_query(variables);

        let res = client.post(&self.base_url).json(&body).send().await?;
        let response: Response<market_query::ResponseData> = res.json().await?;
        let apy = response
            .data
            .unwrap()
            .market
            .unwrap()
            .reserves
            .iter()
            .find(|r| &r.underlying_token.symbol == token)
            .unwrap()
            .supply_info
            .apy
            .clone();

        let rate = LendingRate {
            apy: apy.value.parse()?,
            protocol: "Aave".to_string(),
            token: token.to_string(),
            vault_name: "Aave V3 Pool".to_string(),
        };

        Ok(vec![rate])
    }
}
