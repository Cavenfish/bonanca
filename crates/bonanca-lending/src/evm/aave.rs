use alloy::{
    primitives::{Address, U256, address},
    providers::Provider,
    sol,
};
use anyhow::Result;
use async_trait::async_trait;
use std::str::FromStr;

use crate::traits::Lender;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    PoolV3,
    "src/evm/ABI/aave_pool.json"
}

pub struct AaveV3<P: Provider> {
    pub user: Address,
    pub pool: Address,
    pub client: P,
}

#[async_trait]
impl<P: Provider> Lender for AaveV3<P> {
    async fn supply(&self, token: &str, amount: u64) -> Result<()> {
        let poolv3 = PoolV3::new(self.pool, &self.client);
        let asset = Address::from_str(token)?;

        poolv3
            .supply(asset, U256::from(amount), self.user, 0)
            .send()
            .await?
            .watch()
            .await?;

        Ok(())
    }

    async fn borrow(&self, token: &str, amount: u64) -> Result<()> {
        let poolv3 = PoolV3::new(self.pool, &self.client);
        let asset = Address::from_str(token)?;
        let variable_interest_rate = U256::from(2);

        poolv3
            .borrow(
                asset,
                U256::from(amount),
                variable_interest_rate,
                0,
                self.user,
            )
            .send()
            .await?
            .watch()
            .await?;

        Ok(())
    }

    async fn repay(&self, token: &str, amount: u64) -> Result<()> {
        let poolv3 = PoolV3::new(self.pool, &self.client);
        let asset = Address::from_str(token)?;
        let variable_interest_rate = U256::from(2);

        poolv3
            .repay(asset, U256::from(amount), variable_interest_rate, self.user)
            .send()
            .await?
            .watch()
            .await?;

        Ok(())
    }

    async fn withdraw(&self, token: &str, amount: u64) -> Result<()> {
        let poolv3 = PoolV3::new(self.pool, &self.client);
        let asset = Address::from_str(token)?;

        poolv3
            .withdraw(asset, U256::from(amount), self.user)
            .send()
            .await?
            .watch()
            .await?;

        Ok(())
    }
}

impl<P: Provider> AaveV3<P> {
    pub fn new(chain: &str, pubkey: &str, client: P) -> Self {
        let user = Address::from_str(pubkey).unwrap();
        let pool = match chain {
            "Ethereum" => address!("0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"),
            "Polygon" => address!("0x794a61358D6845594F94dc1DB02A252b5b4814aD"),
            _ => panic!(),
        };

        Self { user, pool, client }
    }

    pub async fn get_pools(&self) -> Result<()> {
        let pool = PoolV3::new(self.pool, &self.client);

        let reserves = pool.getReservesList().call().await?;

        println!("Found {} reserves:\n", reserves.len());

        for reserve_address in reserves.iter() {
            println!("Reserve Address: {}", reserve_address);

            let data = pool.getReserveData(*reserve_address).call().await?;

            println!("\t aToken: {}", data.aTokenAddress);
            println!("\t Variable Debt Token: {}", data.variableDebtTokenAddress);
            println!("\t Liquidity Index: {}", data.liquidityIndex);
            println!(
                "\t Current Liquidity Rate: {}",
                (data.currentLiquidityRate as f64) / 1e27
            );
            println!(
                "\t Current Variable Borrow Rate: {}",
                (data.currentVariableBorrowRate as f64) / 1e27
            );

            println!();
        }

        Ok(())
    }

    pub async fn get_accout_data(&self) -> Result<()> {
        let pool = PoolV3::new(self.pool, &self.client);

        let data = pool.getUserAccountData(self.user).call().await?;

        // Base currency is USD with 8 decimals
        println!("{}", f64::from(data.totalCollateralBase) / 1e8);
        println!("{}", f64::from(data.totalDebtBase) / 1e8);
        println!("{}", f64::from(data.ltv) / 1e4);
        println!("{}", f64::from(data.healthFactor) / 1e18);
        println!("{}", f64::from(data.currentLiquidationThreshold) / 1e4);
        println!("{}", f64::from(data.availableBorrowsBase) / 1e8);

        Ok(())
    }
}
