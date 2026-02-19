use bonanca_wallets::{HdWalletLoad, HdWalletView, wallets::solana::SolWallet};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::path::PathBuf;
use tokio::runtime::Runtime;

#[pyclass(name = "SolWalletView")]
pub struct PySolWalletView {
    inner: SolWallet,
    rt: Runtime,
}

#[pymethods]
impl PySolWalletView {
    #[new]
    fn view(keyvault: PathBuf, rpc: &str, child: u32) -> Self {
        let inner = SolWallet::view(&keyvault, rpc, child);
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

    fn format_native(&self, amount: f64) -> PyResult<u64> {
        self.inner
            .format_native(amount)
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn parse_native(&self, amount: u64) -> PyResult<f64> {
        self.inner
            .parse_native(amount)
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn format_token(&self, amount: f64, token: &str) -> PyResult<u64> {
        self.rt
            .block_on(self.inner.format_token(amount, token))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn parse_token(&self, amount: u64, token: &str) -> PyResult<f64> {
        self.rt
            .block_on(self.inner.parse_token(amount, token))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }
}

#[pyclass(name = "SolWallet")]
pub struct PySolWallet {
    pub inner: SolWallet,
    pub rt: Runtime,
}

#[pymethods]
impl PySolWallet {
    #[new]
    fn load(keyvault: PathBuf, rpc: &str, child: u32) -> Self {
        let inner = SolWallet::load(&keyvault, rpc, child);
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

    fn create_token_account(&self, mint: &str) -> PyResult<String> {
        let result = self
            .rt
            .block_on(self.inner.create_token_account(&mint))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        Ok(result.to_string())
    }

    fn close_token_account(&self, mint: &str) -> PyResult<()> {
        self.rt
            .block_on(self.inner.close_token_account(&mint))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn format_native(&self, amount: f64) -> PyResult<u64> {
        self.inner
            .format_native(amount)
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn parse_native(&self, amount: u64) -> PyResult<f64> {
        self.inner
            .parse_native(amount)
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn format_token(&self, amount: f64, token: &str) -> PyResult<u64> {
        self.rt
            .block_on(self.inner.format_token(amount, token))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }

    fn parse_token(&self, amount: u64, token: &str) -> PyResult<f64> {
        self.rt
            .block_on(self.inner.parse_token(amount, token))
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

    fn transfer_token(&self, mint: &str, amount: f64, to: &str) -> PyResult<()> {
        let _ = self
            .rt
            .block_on(self.inner.transfer_token(mint, amount, to))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }

    fn transfer_all_tokens(&self, mint: &str, to: &str) -> PyResult<()> {
        self.rt
            .block_on(self.inner.transfer_all_tokens(mint, to))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }
}
