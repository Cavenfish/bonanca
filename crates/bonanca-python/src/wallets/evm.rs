use bonanca_wallets::wallets::evm::EvmWallet;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::path::PathBuf;
use tokio::runtime::Runtime;

#[pyclass(name = "EvmWalletView")]
pub struct PyEvmWalletView {
    inner: EvmWallet,
    rt: Runtime,
}

#[pymethods]
impl PyEvmWalletView {
    #[new]
    fn view(keyvault: PathBuf, rpc: &str, child: u32) -> Self {
        let inner = EvmWallet::view(&keyvault, rpc, child);
        let rt = Runtime::new().unwrap();
        Self { inner, rt }
    }

    fn get_pubkey(&self) -> String {
        self.inner.get_pubkey().unwrap()
    }

    fn balance(&self) -> f64 {
        self.rt.block_on(self.inner.balance()).unwrap()
    }

    fn token_balance(&self, token: &str) -> f64 {
        self.rt.block_on(self.inner.token_balance(token)).unwrap()
    }

    fn get_token_allowance(&self, token: &str, spender: &str) -> PyResult<f64> {
        self.rt
            .block_on(self.inner.get_token_allowance(token, spender))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn parse_native_amount(&self, amount: f64) -> PyResult<u64> {
        self.inner
            .parse_native_amount(amount)
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn parse_token_amount(&self, amount: f64, token: &str) -> PyResult<u64> {
        self.rt
            .block_on(self.inner.parse_token_amount(amount, token))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }
}

#[pyclass(name = "EvmWallet")]
pub struct PyEvmWallet {
    pub inner: EvmWallet,
    pub rt: Runtime,
}

#[pymethods]
impl PyEvmWallet {
    #[new]
    fn load(keyvault: PathBuf, rpc: &str, child: u32) -> Self {
        let inner = EvmWallet::load(&keyvault, rpc, child);
        let rt = Runtime::new().unwrap();
        Self { inner, rt }
    }

    fn get_pubkey(&self) -> String {
        self.inner.get_pubkey().unwrap()
    }

    fn balance(&self) -> f64 {
        self.rt.block_on(self.inner.balance()).unwrap()
    }

    fn token_balance(&self, token: &str) -> f64 {
        self.rt.block_on(self.inner.token_balance(token)).unwrap()
    }

    fn approve_token_spending(&self, token: &str, spender: &str, amount: f64) -> PyResult<()> {
        self.rt
            .block_on(self.inner.approve_token_spending(token, spender, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn get_token_allowance(&self, token: &str, spender: &str) -> PyResult<f64> {
        self.rt
            .block_on(self.inner.get_token_allowance(token, spender))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn parse_native_amount(&self, amount: f64) -> PyResult<u64> {
        self.inner
            .parse_native_amount(amount)
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn parse_token_amount(&self, amount: f64, token: &str) -> PyResult<u64> {
        self.rt
            .block_on(self.inner.parse_token_amount(amount, token))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn close(&self, to: &str) -> PyResult<()> {
        self.rt
            .block_on(self.inner.close(to))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn transfer(&self, to: &str, amount: f64) -> PyResult<()> {
        let _ = self
            .rt
            .block_on(self.inner.transfer(to, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }

    fn transfer_token(&self, token: &str, amount: f64, to: &str) -> PyResult<()> {
        let _ = self
            .rt
            .block_on(self.inner.transfer_token(token, amount, to))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }

    fn transfer_all_tokens(&self, token: &str, to: &str) -> PyResult<()> {
        self.rt
            .block_on(self.inner.transfer_all_tokens(token, to))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }
}
