use bonanca_defi::evm::zerox::ZeroX;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use crate::wallets::evm::PyEvmWallet;

#[pyclass(name = "ZeroX")]
pub struct PyZeroX {
    inner: ZeroX,
}

#[pymethods]
impl PyZeroX {
    #[new]
    fn new(api_key: String, chain_id: u16) -> Self {
        let inner = ZeroX::new(api_key, chain_id);
        Self { inner }
    }

    fn swap(&self, wallet: &PyEvmWallet, sell: &str, buy: &str, amount: f64) -> PyResult<()> {
        wallet
            .rt
            .block_on(self.inner.swap(&wallet.inner, sell, buy, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }
}
