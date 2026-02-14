use alloy::{
    primitives::{Address, I32, U256},
    sol,
};
use alloy_primitives::{
    Signed,
    aliases::{I24, U24},
};
use anyhow::Result;
use bonanca_wallets::wallets::evm::EvmWallet;
use std::{ops::Add, str::FromStr};

use crate::evm::uniswap::INonfungiblePositionManager::MintParams;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    Router2,
    "src/evm/ABI/uniswap_v2_router2.json"
}

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    PositionManagerV3,
    "src/evm/ABI/uniswap_v3_nft_position_manager.json"
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
    ) -> Result<Vec<f64>> {
        let router = Router2::new(self.router, &wallet.client);
        let amnt = wallet.format_token(amount_in, path.get(0).unwrap()).await?;

        let amounts = router
            .getAmountsOut(
                U256::from(amnt),
                path.iter().map(|a| Address::from_str(a).unwrap()).collect(),
            )
            .call()
            .await?;

        let mut results = Vec::new();

        for (amount, token) in amounts.iter().zip(path) {
            let result = wallet.parse_token(amount.to::<u64>(), token).await?;
            results.push(result);
        }

        Ok(results)
    }

    pub async fn get_amounts_in(
        &self,
        wallet: &EvmWallet,
        amount_out: f64,
        path: Vec<&str>,
    ) -> Result<Vec<f64>> {
        let router = Router2::new(self.router, &wallet.client);
        let amnt = wallet
            .format_token(amount_out, path.last().unwrap())
            .await?;

        let amounts = router
            .getAmountsIn(
                U256::from(amnt),
                path.iter().map(|a| Address::from_str(a).unwrap()).collect(),
            )
            .call()
            .await?;

        let mut results = Vec::new();

        for (amount, token) in amounts.iter().zip(path) {
            let result = wallet.parse_token(amount.to::<u64>(), token).await?;
            results.push(result);
        }

        Ok(results)
    }

    pub async fn add_liquidity(
        &self,
        wallet: &EvmWallet,
        token_a: &str,
        token_b: &str,
        amount_a_desired: f64,
        amount_b_desired: f64,
        amount_a_min: f64,
        amount_b_min: f64,
        to: &str,
        deadline: u64,
    ) -> Result<(f64, f64, u128)> {
        let router = Router2::new(self.router, &wallet.client);
        let token_a_addr = Address::from_str(token_a)?;
        let token_b_addr = Address::from_str(token_b)?;
        let to_addr = Address::from_str(to)?;

        let amount_a = wallet.format_token(amount_a_desired, token_a).await?;
        let amount_b = wallet.format_token(amount_b_desired, token_b).await?;
        let amount_a_min_fmt = wallet.format_token(amount_a_min, token_a).await?;
        let amount_b_min_fmt = wallet.format_token(amount_b_min, token_b).await?;

        let result = router
            .addLiquidity(
                token_a_addr,
                token_b_addr,
                U256::from(amount_a),
                U256::from(amount_b),
                U256::from(amount_a_min_fmt),
                U256::from(amount_b_min_fmt),
                to_addr,
                U256::from(deadline),
            )
            .call()
            .await?;

        let amount_a_out = wallet
            .parse_token(result.amountA.to::<u64>(), token_a)
            .await?;
        let amount_b_out = wallet
            .parse_token(result.amountB.to::<u64>(), token_b)
            .await?;

        Ok((amount_a_out, amount_b_out, result.liquidity.to::<u128>()))
    }

    pub async fn remove_liquidity(
        &self,
        wallet: &EvmWallet,
        token_a: &str,
        token_b: &str,
        liquidity: u128,
        amount_a_min: f64,
        amount_b_min: f64,
        to: &str,
        deadline: u64,
    ) -> Result<(f64, f64)> {
        let router = Router2::new(self.router, &wallet.client);
        let token_a_addr = Address::from_str(token_a)?;
        let token_b_addr = Address::from_str(token_b)?;
        let to_addr = Address::from_str(to)?;

        let amount_a_min_fmt = wallet.format_token(amount_a_min, token_a).await?;
        let amount_b_min_fmt = wallet.format_token(amount_b_min, token_b).await?;

        let result = router
            .removeLiquidity(
                token_a_addr,
                token_b_addr,
                U256::from(liquidity),
                U256::from(amount_a_min_fmt),
                U256::from(amount_b_min_fmt),
                to_addr,
                U256::from(deadline),
            )
            .call()
            .await?;

        let amount_a_out = wallet
            .parse_token(result.amountA.to::<u64>(), token_a)
            .await?;
        let amount_b_out = wallet
            .parse_token(result.amountB.to::<u64>(), token_b)
            .await?;

        Ok((amount_a_out, amount_b_out))
    }

    pub async fn add_liquidity_eth(
        &self,
        wallet: &EvmWallet,
        token: &str,
        amount_token_desired: f64,
        amount_token_min: f64,
        amount_eth_min: f64,
        to: &str,
        deadline: u64,
    ) -> Result<(f64, f64, u128)> {
        let router = Router2::new(self.router, &wallet.client);
        let token_addr = Address::from_str(token)?;
        let to_addr = Address::from_str(to)?;

        let amount_token = wallet.format_token(amount_token_desired, token).await?;
        let amount_token_min_fmt = wallet.format_token(amount_token_min, token).await?;
        let amount_eth_min_fmt = wallet.format_token(amount_eth_min, "ETH").await?;

        let result = router
            .addLiquidityETH(
                token_addr,
                U256::from(amount_token),
                U256::from(amount_token_min_fmt),
                U256::from(amount_eth_min_fmt),
                to_addr,
                U256::from(deadline),
            )
            .call()
            .await?;

        let amount_token_out = wallet
            .parse_token(result.amountToken.to::<u64>(), token)
            .await?;
        let amount_eth_out = wallet
            .parse_token(result.amountETH.to::<u64>(), "ETH")
            .await?;

        Ok((
            amount_token_out,
            amount_eth_out,
            result.liquidity.to::<u128>(),
        ))
    }

    pub async fn remove_liquidity_eth(
        &self,
        wallet: &EvmWallet,
        token: &str,
        liquidity: u128,
        amount_token_min: f64,
        amount_eth_min: f64,
        to: &str,
        deadline: u64,
    ) -> Result<(f64, f64)> {
        let router = Router2::new(self.router, &wallet.client);
        let token_addr = Address::from_str(token)?;
        let to_addr = Address::from_str(to)?;

        let amount_token_min_fmt = wallet.format_token(amount_token_min, token).await?;
        let amount_eth_min_fmt = wallet.format_token(amount_eth_min, "ETH").await?;

        let result = router
            .removeLiquidityETH(
                token_addr,
                U256::from(liquidity),
                U256::from(amount_token_min_fmt),
                U256::from(amount_eth_min_fmt),
                to_addr,
                U256::from(deadline),
            )
            .call()
            .await?;

        let amount_token_out = wallet
            .parse_token(result.amountToken.to::<u64>(), token)
            .await?;
        let amount_eth_out = wallet
            .parse_token(result.amountETH.to::<u64>(), "ETH")
            .await?;

        Ok((amount_token_out, amount_eth_out))
    }
}

pub struct UniswapV3 {
    manager: Address,
}

impl UniswapV3 {
    pub fn new(manager: &str) -> Self {
        Self {
            manager: Address::from_str(manager).unwrap(),
        }
    }

    pub async fn mint(
        &self,
        wallet: &EvmWallet,
        token0: &str,
        token1: &str,
        fee: u32,
        tick_lower: i32,
        tick_upper: i32,
        amount0_desired: f64,
        amount1_desired: f64,
        amount0_min: f64,
        amount1_min: f64,
        recipient: &str,
        deadline: u64,
    ) -> Result<(u64, u128, f64, f64)> {
        let pmv3 = PositionManagerV3::new(self.manager, &wallet.client);
        let token0_addr = Address::from_str(token0)?;
        let token1_addr = Address::from_str(token1)?;
        let recipient_addr = Address::from_str(recipient)?;

        let amt0_desired = wallet.format_token(amount0_desired, token0).await?;
        let amt1_desired = wallet.format_token(amount1_desired, token1).await?;
        let amt0_min = wallet.format_token(amount0_min, token0).await?;
        let amt1_min = wallet.format_token(amount1_min, token1).await?;

        let params = MintParams {
            token0: token0_addr,
            token1: token1_addr,
            fee: U24::try_from(fee)?,
            tickLower: I24::try_from(tick_lower)?,
            tickUpper: I24::try_from(tick_upper)?,
            amount0Desired: U256::from(amt0_desired),
            amount1Desired: U256::from(amt1_desired),
            amount0Min: U256::from(amt0_min),
            amount1Min: U256::from(amt1_min),
            recipient: recipient_addr,
            deadline: U256::from(deadline),
        };

        let result = pmv3.mint(params).call().await?;

        let amount0_out = wallet
            .parse_token(result.amount0.to::<u64>(), token0)
            .await?;
        let amount1_out = wallet
            .parse_token(result.amount1.to::<u64>(), token1)
            .await?;

        Ok((
            result.tokenId.to::<u64>(),
            result.liquidity,
            amount0_out,
            amount1_out,
        ))
    }
}
