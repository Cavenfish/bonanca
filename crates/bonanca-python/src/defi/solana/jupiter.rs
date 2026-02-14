use bonanca_api_lib::defi::jupiter::{JupiterLendMarket, JupiterSwapQuote};
use bonanca_defi::solana::jupiter::Jupiter;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::time::Duration;

use crate::wallets::solana::PySolWallet;

#[pyclass(name = "JupiterSwapQuote")]
pub struct PyJupiterSwapQuote {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u32,
    pub price_impact_pct: String,
    pub context_slot: u64,
    pub time_taken: f64,
    pub swap_usd_value: String,
}

impl PyJupiterSwapQuote {
    fn from_rust(quote: JupiterSwapQuote) -> Self {
        Self {
            input_mint: quote.input_mint,
            in_amount: quote.in_amount,
            output_mint: quote.output_mint,
            out_amount: quote.out_amount,
            other_amount_threshold: quote.other_amount_threshold,
            swap_mode: quote.swap_mode,
            slippage_bps: quote.slippage_bps,
            price_impact_pct: quote.price_impact_pct,
            context_slot: quote.context_slot,
            time_taken: quote.time_taken,
            swap_usd_value: quote.swap_usd_value,
        }
    }
}

#[pyclass(name = "SolTxnReceipt")]
#[allow(dead_code)]
pub struct PySolTxnReceipt {
    pub hash: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub gas_used: f64,
}

impl PySolTxnReceipt {
    fn from_rust(receipt: bonanca_wallets::wallets::solana::SolTxnReceipt) -> Self {
        Self {
            hash: receipt.hash,
            slot: receipt.slot,
            block_time: receipt.block_time,
            gas_used: receipt.gas_used,
        }
    }
}

#[pyclass(name = "JupiterLendMarket")]
#[allow(dead_code)]
pub struct PyJupiterLendMarket {
    pub id: u64,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub asset_address: String,
    pub total_assets: String,
    pub total_supply: String,
    pub convert_to_shares: String,
    pub convert_to_assets: String,
    pub rewards_rate: String,
    pub supply_rate: String,
    pub total_rate: String,
}

impl PyJupiterLendMarket {
    fn from_rust(market: JupiterLendMarket) -> Self {
        Self {
            id: market.id,
            address: market.address,
            name: market.name,
            symbol: market.symbol,
            decimals: market.decimals,
            asset_address: market.asset_address,
            total_assets: market.total_assets,
            total_supply: market.total_supply,
            convert_to_shares: market.convert_to_shares,
            convert_to_assets: market.convert_to_assets,
            rewards_rate: market.rewards_rate,
            supply_rate: market.supply_rate,
            total_rate: market.total_rate,
        }
    }
}

#[pyclass(name = "Jupiter")]
pub struct PyJupiter {
    inner: Jupiter,
}

#[pymethods]
impl PyJupiter {
    #[new]
    fn new(api_key: String) -> Self {
        let inner = Jupiter::new(api_key);
        Self { inner }
    }

    fn get_token_price(&self, wallet: &PySolWallet, token: &str, amount: f64) -> PyResult<f64> {
        wallet
            .rt
            .block_on(self.inner.get_token_price(token, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn get_swap_quote(
        &self,
        wallet: &PySolWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> PyResult<PyJupiterSwapQuote> {
        let quote = wallet
            .rt
            .block_on(self.inner.get_swap_quote(&wallet.inner, sell, buy, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(PyJupiterSwapQuote::from_rust(quote))
    }

    fn swap(&self, wallet: &PySolWallet, quote: &PyJupiterSwapQuote) -> PyResult<PySolTxnReceipt> {
        // Reconstruct the JupiterSwapQuote from the Python wrapper
        let rust_quote = JupiterSwapQuote {
            input_mint: quote.input_mint.clone(),
            in_amount: quote.in_amount.clone(),
            output_mint: quote.output_mint.clone(),
            out_amount: quote.out_amount.clone(),
            other_amount_threshold: quote.other_amount_threshold.clone(),
            swap_mode: quote.swap_mode.clone(),
            slippage_bps: quote.slippage_bps,
            platform_fee: None,
            price_impact_pct: quote.price_impact_pct.clone(),
            route_plan: vec![],
            context_slot: quote.context_slot,
            time_taken: quote.time_taken,
            swap_usd_value: quote.swap_usd_value.clone(),
            simpler_route_used: false,
            use_incurred_slippage_for_quoting: None,
            other_route_plans: None,
            loaded_longtail_token: false,
            instruction_version: None,
        };

        let receipt = wallet
            .rt
            .block_on(self.inner.swap(&wallet.inner, rust_quote))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(PySolTxnReceipt::from_rust(receipt))
    }

    fn quick_swap(
        &self,
        wallet: &PySolWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> PyResult<PySolTxnReceipt> {
        let receipt = wallet
            .rt
            .block_on(self.inner.quick_swap(&wallet.inner, sell, buy, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(PySolTxnReceipt::from_rust(receipt))
    }

    fn limit_order(
        &self,
        wallet: &PySolWallet,
        sell: &str,
        buy: &str,
        sell_amount: f64,
        buy_amount: f64,
        lifetime_secs: u64,
    ) -> PyResult<PySolTxnReceipt> {
        let receipt = wallet
            .rt
            .block_on(self.inner.limit_order(
                &wallet.inner,
                sell,
                buy,
                sell_amount,
                buy_amount,
                Duration::from_secs(lifetime_secs),
            ))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(PySolTxnReceipt::from_rust(receipt))
    }

    fn limit_order_by_price(
        &self,
        wallet: &PySolWallet,
        sell: &str,
        buy: &str,
        amount: f64,
        price: f64,
        lifetime_secs: u64,
    ) -> PyResult<PySolTxnReceipt> {
        let receipt = wallet
            .rt
            .block_on(self.inner.limit_order(
                &wallet.inner,
                sell,
                buy,
                amount,
                price,
                Duration::from_secs(lifetime_secs),
            ))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(PySolTxnReceipt::from_rust(receipt))
    }

    fn get_lendable_tokens(&self, wallet: &PySolWallet) -> PyResult<Vec<PyJupiterLendMarket>> {
        let markets = wallet
            .rt
            .block_on(self.inner.get_lendable_tokens())
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(markets
            .into_iter()
            .map(PyJupiterLendMarket::from_rust)
            .collect())
    }

    fn deposit(&self, wallet: &PySolWallet, token: &str, amount: f64) -> PyResult<PySolTxnReceipt> {
        let receipt = wallet
            .rt
            .block_on(self.inner.deposit(&wallet.inner, token, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(PySolTxnReceipt::from_rust(receipt))
    }

    fn withdraw(
        &self,
        wallet: &PySolWallet,
        token: &str,
        amount: f64,
    ) -> PyResult<PySolTxnReceipt> {
        let receipt = wallet
            .rt
            .block_on(self.inner.withdraw(&wallet.inner, token, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(PySolTxnReceipt::from_rust(receipt))
    }
}
