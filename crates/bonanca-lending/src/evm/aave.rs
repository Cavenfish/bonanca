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
}
