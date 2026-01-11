use bonanca_defi::evm::morpho::MorphoVaultV1;
use pyo3::prelude::*;

use crate::wallets::evm::PyEvmWallet;

#[pyclass(name = "MorphoVaultV1")]
pub struct PyMorphoVaultV1 {
    inner: MorphoVaultV1,
}

#[pymethods]
impl PyMorphoVaultV1 {
    #[new]
    fn new() -> Self {
        let inner = MorphoVaultV1::new();
        Self { inner }
    }

    fn get_user_data(&self, user: &str, chain_id: i64, wallet: &PyEvmWallet) -> PyResult<String> {
        let user_data = wallet
            .rt
            .block_on(self.inner.get_user_data(user, chain_id))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(format!("{:#?}", user_data))
    }

    fn get_token_vaults(
        &self,
        token_symbol: &str,
        chain_id: i64,
        wallet: &PyEvmWallet,
    ) -> PyResult<String> {
        let vaults = wallet
            .rt
            .block_on(self.inner.get_token_vaults(token_symbol, chain_id))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(format!("{:#?}", vaults))
    }

    fn supply(&self, wallet: &PyEvmWallet, vault_address: &str, amount: f64) -> PyResult<()> {
        wallet
            .rt
            .block_on(self.inner.supply(&wallet.inner, vault_address, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn withdraw(&self, wallet: &PyEvmWallet, vault_address: &str, amount: f64) -> PyResult<()> {
        wallet
            .rt
            .block_on(self.inner.withdraw(&wallet.inner, vault_address, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}
