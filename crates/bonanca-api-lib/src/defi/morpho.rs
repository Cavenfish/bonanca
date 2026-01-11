use alloy_primitives::Address;
use anyhow::Result;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/morpho/schema.graphql",
    query_path = "schemas/morpho/query.graphql",
    response_derives = "Debug"
)]
pub struct VaultsV1Query;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/morpho/schema.graphql",
    query_path = "schemas/morpho/query.graphql",
    response_derives = "Debug"
)]
pub struct UserDataQuery;

pub struct MorphoApi {
    pub base_url: String,
}

impl MorphoApi {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.morpho.org/graphql".to_string(),
        }
    }

    pub async fn query_user_data(
        &self,
        user: &str,
        chain_id: i64,
    ) -> Result<user_data_query::UserDataQueryUserByAddress> {
        let client = Client::new();

        let variables = user_data_query::Variables {
            address: user.to_string(),
            chain_id: chain_id,
        };
        let body = UserDataQuery::build_query(variables);

        let res = client.post(&self.base_url).json(&body).send().await?;
        let response: Response<user_data_query::ResponseData> = res.json().await?;
        let data = response.data.unwrap().user_by_address;

        Ok(data)
    }

    pub async fn query_vaults_v1(
        &self,
        token_symbol: &str,
        chain_id: i64,
    ) -> Result<Vec<vaults_v1_query::VaultsV1QueryVaultsItems>> {
        let client = Client::new();

        let variables = vaults_v1_query::Variables {
            first: 15,
            chain_id: chain_id,
            asset: token_symbol.to_string(),
        };
        let body = VaultsV1Query::build_query(variables);

        let res = client.post(&self.base_url).json(&body).send().await?;
        let response: Response<vaults_v1_query::ResponseData> = res.json().await?;
        let vaults = response.data.unwrap().vaults.items.unwrap();

        Ok(vaults)
    }
}
