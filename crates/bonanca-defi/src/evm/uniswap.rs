use alloy::{
    primitives::{Address, U256},
    providers::DynProvider,
    rpc::types::TransactionReceipt,
    sol,
};
use alloy_primitives::address;
use anyhow::Result;
use bonanca_wallets::wallets::evm::EvmWallet;
use std::str::FromStr;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    Router2,
    "src/evm/ABI/uniswap_v2_router2.json"
}

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    PoolV3,
    "src/evm/ABI/uniswap_v3_pool.json"
}

pub struct UniswapV2 {
    router: Address,
}

impl UniswapV2 {
    pub fn new(router: &str) -> Self {
        Self {
            router: Address::from_str(router).expect("Not a proper eth address"),
        }
    }

    pub async fn get_amounts_out(
        &self,
        wallet: &EvmWallet,
        amount_in: f64,
        path: Vec<&str>,
    ) -> Result<f64> {
        let router = Router2::new(self.router, &wallet.client);
        let amnt = wallet
            .parse_token_amount(amount_in, path.get(0).unwrap())
            .await?;

        router
            .getAmountsOut(
                U256::from(amnt),
                path.iter().map(|a| Address::from_str(a).unwrap()).collect(),
            )
            .call()
            .await?;

        Ok(1.5)
    }
}
