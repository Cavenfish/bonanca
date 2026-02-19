use alloy::rpc::types::TransactionReceipt;
use bonanca_wallets::{HdWalletLoad, HdWalletView, wallets::evm::EvmWallet};
use pyo3::prelude::*;
use pyo3::{exceptions::PyRuntimeError, types::PyDict};
use std::path::PathBuf;
use tokio::runtime::Runtime;

pub fn parse_txn_receipt<'py>(
    py: Python<'py>,
    receipt: TransactionReceipt,
) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);

    dict.set_item("transaction_hash", receipt.transaction_hash.to_string())?;
    dict.set_item("gas_used", receipt.gas_used)?;
    dict.set_item("effective_gas_price", receipt.effective_gas_price)?;
    dict.set_item("from", receipt.from.to_string())?;

    if let Some(block_hash) = receipt.block_hash {
        dict.set_item("block_hash", block_hash.to_string())?;
    }

    if let Some(block_number) = receipt.block_number {
        dict.set_item("block_number", block_number)?;
    }
    if let Some(gas_price) = receipt.blob_gas_price {
        dict.set_item("gas_price", gas_price)?;
    }

    if let Some(to) = receipt.to {
        dict.set_item("to", to.to_string())?;
    }

    Ok(dict.into())
}

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

    fn transfer<'py>(&self, py: Python<'py>, to: &str, amount: f64) -> PyResult<Py<PyDict>> {
        let receipt = self
            .rt
            .block_on(self.inner.transfer(to, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        parse_txn_receipt(py, receipt)
    }

    fn transfer_token<'py>(
        &self,
        py: Python<'py>,
        token: &str,
        amount: f64,
        to: &str,
    ) -> PyResult<Py<PyDict>> {
        let receipt = self
            .rt
            .block_on(self.inner.transfer_token(token, amount, to))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        parse_txn_receipt(py, receipt)
    }

    fn transfer_all_tokens(&self, token: &str, to: &str) -> PyResult<()> {
        self.rt
            .block_on(self.inner.transfer_all_tokens(token, to))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
    }
}
