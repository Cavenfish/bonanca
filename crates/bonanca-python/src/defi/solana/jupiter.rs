use bonanca_api_lib::defi::jupiter::JupiterSwapQuote;
use bonanca_defi::solana::jupiter::Jupiter;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

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
}
