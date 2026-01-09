use bonanca_oracle::prices::{CoinMarketCap, DefiLlama};
use pyo3::prelude::*;
use tokio::runtime::Runtime;

#[pyclass(name = "CoinMarketCap")]
pub struct PyCoinMarketCap {
    inner: CoinMarketCap,
    rt: Runtime,
}

#[pymethods]
impl PyCoinMarketCap {
    #[new]
    fn new(api_key: String) -> Self {
        let inner = CoinMarketCap::new(api_key);
        let rt = Runtime::new().unwrap();
        Self { inner, rt }
    }

    fn get_token_price(&self, symbol: &str, amount: f64) -> PyResult<f64> {
        self.rt
            .block_on(self.inner.get_token_price(symbol, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

#[pyclass(name = "DefiLlama")]
pub struct PyDefiLlama {
    inner: DefiLlama,
    rt: Runtime,
}

#[pymethods]
impl PyDefiLlama {
    #[new]
    fn new() -> Self {
        let inner = DefiLlama::new();
        let rt = Runtime::new().unwrap();
        Self { inner, rt }
    }

    fn get_token_price(&self, token: &str, amount: f64, chain: &str) -> PyResult<f64> {
        self.rt
            .block_on(self.inner.get_token_price(token, amount, chain))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}
