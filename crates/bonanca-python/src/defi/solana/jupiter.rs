use bonanca_defi::solana::jupiter::Jupiter;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use crate::wallets::solana::PySolWallet;

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

    fn swap(&self, wallet: &PySolWallet, sell: &str, buy: &str, amount: f64) -> PyResult<()> {
        wallet
            .rt
            .block_on(self.inner.swap(&wallet.inner, sell, buy, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }
}
