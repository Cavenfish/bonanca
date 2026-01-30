use bonanca_defi::evm::cow::CoW;
use pyo3::prelude::*;
use std::time::Duration;

use crate::wallets::evm::PyEvmWallet;

#[pyclass(name = "CoW")]
pub struct PyCoW {
    inner: CoW,
}

#[pymethods]
impl PyCoW {
    #[new]
    fn new(chain: &str) -> PyResult<Self> {
        let inner = CoW::new(chain)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { inner })
    }

    fn market_order(
        &self,
        wallet: &PyEvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> PyResult<String> {
        let quote = wallet
            .rt
            .block_on(
                self.inner
                    .get_market_quote(&wallet.inner, sell, buy, amount),
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        wallet
            .rt
            .block_on(self.inner.post_market_order(&wallet.inner, quote))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn limit_order(
        &self,
        wallet: &PyEvmWallet,
        sell: &str,
        buy: &str,
        sell_amount: f64,
        buy_amount: f64,
        lifetime: (u64, u64),
    ) -> PyResult<String> {
        let lifetime = Duration::from_hours(lifetime.0) + Duration::from_mins(lifetime.1);
        wallet
            .rt
            .block_on(self.inner.limit_order(
                &wallet.inner,
                sell,
                buy,
                sell_amount,
                buy_amount,
                lifetime,
            ))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn limit_order_by_price(
        &self,
        wallet: &PyEvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
        sell_price: f64,
        lifetime: (u64, u64),
    ) -> PyResult<String> {
        let lifetime = Duration::from_hours(lifetime.0) + Duration::from_mins(lifetime.1);
        wallet
            .rt
            .block_on(self.inner.limit_order_by_price(
                &wallet.inner,
                sell,
                buy,
                amount,
                sell_price,
                lifetime,
            ))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}
