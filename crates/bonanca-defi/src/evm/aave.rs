use alloy::{
    primitives::{Address, U256},
    providers::DynProvider,
    rpc::types::TransactionReceipt,
    sol,
};
use anyhow::Result;
use bonanca_api_lib::defi::aave::{AaveV3Api, AaveV3ReserveData};
use bonanca_wallets::wallets::evm::EvmWallet;
use std::str::FromStr;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    PoolV3,
    "src/evm/ABI/aave_pool.json"
}

pub struct AaveV3 {
    api: AaveV3Api,
    pub pool: Address,
}

impl AaveV3 {
    pub fn new(chain_id: u64) -> Self {
        let api = AaveV3Api::new();
        let pool = api.get_pool_address(chain_id).unwrap();

        Self { api, pool }
    }

    pub async fn get_pools_api(&self, chain_id: u64) -> Result<Vec<AaveV3ReserveData>> {
        let data = self.api.query_market(chain_id).await.unwrap();

        Ok(data)
    }

    pub async fn get_pools(
        &self,
        client: &DynProvider,
    ) -> Result<Vec<DataTypes::ReserveDataLegacy>> {
        let pool = PoolV3::new(self.pool, client);

        let reserves = pool.getReservesList().call().await?;

        let mut reserve_data: Vec<DataTypes::ReserveDataLegacy> = Vec::new();

        for reserve_address in reserves.iter() {
            let data = pool.getReserveData(*reserve_address).call().await?;

            reserve_data.push(data);
        }

        Ok(reserve_data)
    }

    pub async fn get_user_data(&self, user: &str, client: &DynProvider) -> Result<AaveV3UserData> {
        let pool = PoolV3::new(self.pool, client);
        let addy = Address::from_str(user)?;

        let data = pool.getUserAccountData(addy).call().await?;

        Ok(AaveV3UserData::new(data))
    }

    pub async fn get_reserve_data(
        &self,
        token: &str,
        client: &DynProvider,
    ) -> Result<DataTypes::ReserveDataLegacy> {
        let pool = PoolV3::new(self.pool, client);
        let asset = Address::from_str(token)?;

        let token_pool = pool.getReserveData(asset).call().await?;

        Ok(token_pool)
    }

    pub async fn supply(
        &self,
        wallet: &EvmWallet,
        token: &str,
        amount: f64,
    ) -> Result<TransactionReceipt> {
        let poolv3 = PoolV3::new(self.pool, &wallet.client);
        let asset = Address::from_str(token)?;
        let amnt = wallet.parse_token_amount(amount, token).await?;

        let sig = poolv3
            .supply(asset, U256::from(amnt), wallet.pubkey, 0)
            .send()
            .await?
            .get_receipt()
            .await?;

        Ok(sig)
    }

    pub async fn borrow(
        &self,
        wallet: &EvmWallet,
        token: &str,
        amount: f64,
    ) -> Result<TransactionReceipt> {
        let poolv3 = PoolV3::new(self.pool, &wallet.client);
        let asset = Address::from_str(token)?;
        let variable_interest_rate = U256::from(2);
        let amnt = wallet.parse_token_amount(amount, token).await?;

        let sig = poolv3
            .borrow(
                asset,
                U256::from(amnt),
                variable_interest_rate,
                0,
                wallet.pubkey,
            )
            .send()
            .await?
            .get_receipt()
            .await?;

        Ok(sig)
    }

    pub async fn repay(
        &self,
        wallet: &EvmWallet,
        token: &str,
        amount: f64,
    ) -> Result<TransactionReceipt> {
        let poolv3 = PoolV3::new(self.pool, &wallet.client);
        let asset = Address::from_str(token)?;
        let variable_interest_rate = U256::from(2);
        let amnt = wallet.parse_token_amount(amount, token).await?;

        let sig = poolv3
            .repay(
                asset,
                U256::from(amnt),
                variable_interest_rate,
                wallet.pubkey,
            )
            .send()
            .await?
            .get_receipt()
            .await?;

        Ok(sig)
    }

    pub async fn withdraw(
        &self,
        wallet: &EvmWallet,
        token: &str,
        amount: f64,
    ) -> Result<TransactionReceipt> {
        let poolv3 = PoolV3::new(self.pool, &wallet.client);
        let asset = Address::from_str(token)?;
        let amnt = wallet.parse_token_amount(amount, token).await?;

        let sig = poolv3
            .withdraw(asset, U256::from(amnt), wallet.pubkey)
            .send()
            .await?
            .get_receipt()
            .await?;

        Ok(sig)
    }
}

pub struct AaveV3UserData {
    pub total_collateral: f64,
    pub total_debt: f64,
    pub ltv: f64,
    pub health_factor: f64,
    pub liquidation_threshold: f64,
    pub available_borrows: f64,
}

impl AaveV3UserData {
    fn new(data: PoolV3::getUserAccountDataReturn) -> Self {
        // Base currency is USD with 8 decimals
        Self {
            total_collateral: f64::from(data.totalCollateralBase) / 1e8,
            total_debt: f64::from(data.totalDebtBase) / 1e8,
            ltv: f64::from(data.ltv) / 1e4,
            health_factor: f64::from(data.healthFactor) / 1e18,
            liquidation_threshold: f64::from(data.currentLiquidationThreshold) / 1e4,
            available_borrows: f64::from(data.availableBorrowsBase) / 1e8,
        }
    }
}
