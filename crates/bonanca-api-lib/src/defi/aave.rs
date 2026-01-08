use alloy_primitives::{Address, ChainId, address};
use anyhow::Result;
use bincode::de;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/aave/schema.graphql",
    query_path = "schemas/aave/query.graphql",
    response_derives = "Debug, Clone"
)]
pub struct MarketQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/aave/schema.graphql",
    query_path = "schemas/aave/query.graphql",
    response_derives = "Debug, Clone"
)]
pub struct MarketsQuery;

pub struct AaveV3Api {
    base_url: String,
}

impl AaveV3Api {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.v3.aave.com/graphql".to_string(),
        }
    }

    pub fn get_pool_address(&self, chain_id: u64) -> Result<Address> {
        let addy = match chain_id {
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

        Ok(addy)
    }

    pub async fn query_market(&self, chain_id: u64) -> Result<Vec<AaveV3ReserveData>> {
        let client = Client::new();
        let address = self.get_pool_address(chain_id)?;

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
        let data = response
            .data
            .unwrap()
            .market
            .unwrap()
            .reserves
            .iter()
            .map(|r| AaveV3ReserveData::new(r))
            .collect();

        Ok(data)
    }

    pub async fn query_markets(&self, chain_ids: Vec<u64>) -> Result<markets_query::ResponseData> {
        let client = Client::new();

        let markets_vars = markets_query::MarketsRequest {
            chain_ids,
            user: None,
        };

        let variables = markets_query::Variables {
            request: markets_vars,
        };

        let body = MarketsQuery::build_query(variables);

        let res = client.post(&self.base_url).json(&body).send().await?;
        let response: Response<markets_query::ResponseData> = res.json().await?;
        let data = response.data.unwrap();

        Ok(data)
    }
}

#[derive(Debug, Clone)]
pub struct AaveV3ReserveData {
    pub is_frozen: bool,
    pub is_paused: bool,
    pub a_token: Token,
    pub underlying_token: Token,
    pub supply_info: SupplyInfo,
    pub borrow_info: Option<BorrowInfo>,
}

impl AaveV3ReserveData {
    fn new(reserve: &market_query::MarketQueryMarketReserves) -> Self {
        let a_token = Token {
            address: reserve.a_token.address.to_string(),
            name: reserve.a_token.name.clone(),
            symbol: reserve.a_token.symbol.clone(),
        };

        let underlying_token = Token {
            address: reserve.underlying_token.address.to_string(),
            name: reserve.underlying_token.name.clone(),
            symbol: reserve.underlying_token.symbol.clone(),
        };

        let supply_info = SupplyInfo {
            can_be_collateral: reserve.supply_info.can_be_collateral,
            supply_cap_reached: reserve.supply_info.supply_cap_reached,
            supply_apy: reserve.supply_info.apy.value.parse().unwrap(),
            max_ltv: reserve.supply_info.max_ltv.value.parse().unwrap(),
            supply_cap: reserve.supply_info.supply_cap.amount.value.parse().unwrap(),
        };

        let borrow_info = match &reserve.borrow_info {
            Some(info) => Some(BorrowInfo::new(&info)),
            None => None,
        };

        Self {
            is_frozen: reserve.is_frozen,
            is_paused: reserve.is_paused,
            a_token,
            underlying_token,
            supply_info,
            borrow_info,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub struct SupplyInfo {
    pub can_be_collateral: bool,
    pub supply_cap_reached: bool,
    pub supply_apy: f64,
    pub max_ltv: f64,
    pub supply_cap: f64,
}

#[derive(Debug, Clone)]
pub struct BorrowInfo {
    pub borrow_cap_reached: bool,
    pub borrow_apy: f64,
    pub borrow_cap: f64,
}

impl BorrowInfo {
    fn new(info: &market_query::MarketQueryMarketReservesBorrowInfo) -> Self {
        Self {
            borrow_cap_reached: info.borrow_cap_reached,
            borrow_apy: info.apy.value.parse().unwrap(),
            borrow_cap: info.borrow_cap.amount.value.parse().unwrap(),
        }
    }
}
