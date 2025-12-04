use alloy_primitives::Address;
use anyhow::Result;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;

use crate::lending_oracle::LendingRate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/morpho/schema.graphql",
    query_path = "schemas/morpho/query.graphql",
    response_derives = "Debug"
)]
pub struct VaultsV1Query;

pub struct MorphoApi {
    pub base_url: String,
}

impl MorphoApi {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.morpho.org/graphql".to_string(),
        }
    }

    pub async fn query_vaults_v1(&self, token: &str, chain_id: u64) -> Result<Vec<LendingRate>> {
        let client = Client::new();

        let variables = vaults_v1_query::Variables {
            first: 5,
            chain_id: chain_id as i64,
            asset: token.to_string(),
        };
        let body = VaultsV1Query::build_query(variables);

        let res = client.post(&self.base_url).json(&body).send().await?;
        let response: Response<vaults_v1_query::ResponseData> = res.json().await?;
        let vaults = response.data.unwrap().vaults.items.unwrap();

        let mut rates: Vec<LendingRate> = Vec::new();

        for vault in vaults.iter() {
            rates.push(LendingRate {
                apy: vault.state.as_ref().unwrap().apy,
                protocol: "Morpho".to_string(),
                token: token.to_string(),
                vault_name: vault.name.clone(),
            });
        }

        Ok(rates)
    }
}
