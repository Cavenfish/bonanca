use bonanca_defi::evm::aave::AaveV3;
use pyo3::{prelude::*, types::PyDict};

use crate::wallets::evm::{PyEvmWallet, parse_txn_receipt};

#[pyclass(name = "AaveV3")]
pub struct PyAaveV3 {
    inner: AaveV3,
}

#[pymethods]
impl PyAaveV3 {
    #[new]
    fn new(chain_id: u64) -> Self {
        let inner = AaveV3::new(chain_id);
        Self { inner }
    }

    fn get_user_data<'py>(
        &self,
        py: Python<'py>,
        user: &str,
        wallet: &PyEvmWallet,
    ) -> PyResult<Py<PyDict>> {
        let result = wallet
            .rt
            .block_on(self.inner.get_user_data(user, &wallet.inner.client))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        let dict = PyDict::new(py);

        dict.set_item("Total Collateral", result.total_collateral)?;
        dict.set_item("Total Debt", result.total_debt)?;
        dict.set_item("Available Borrows", result.available_borrows)?;
        dict.set_item("Liquidation Threshold", result.liquidation_threshold)?;
        dict.set_item("Ltv", result.ltv)?;
        dict.set_item("Health Factor", result.health_factor)?;

        Ok(dict.into())
    }

    // fn get_reserve_data(&self, token: &str, wallet: &PyEvmWallet) -> PyResult<String> {
    //     let result = wallet
    //         .rt
    //         .block_on(self.inner.get_reserve_data(token, &wallet.inner.client))
    //         .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    //     Ok(serde_json::to_string(&result).unwrap())
    // }

    fn supply<'py>(
        &self,
        py: Python<'py>,
        wallet: &PyEvmWallet,
        token: &str,
        amount: f64,
    ) -> PyResult<Py<PyDict>> {
        let receipt = wallet
            .rt
            .block_on(self.inner.supply(&wallet.inner, token, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        parse_txn_receipt(py, receipt)
    }

    fn borrow<'py>(
        &self,
        py: Python<'py>,
        wallet: &PyEvmWallet,
        token: &str,
        amount: f64,
    ) -> PyResult<Py<PyDict>> {
        let receipt = wallet
            .rt
            .block_on(self.inner.borrow(&wallet.inner, token, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        parse_txn_receipt(py, receipt)
    }

    fn repay<'py>(
        &self,
        py: Python<'py>,
        wallet: &PyEvmWallet,
        token: &str,
        amount: f64,
    ) -> PyResult<Py<PyDict>> {
        let receipt = wallet
            .rt
            .block_on(self.inner.repay(&wallet.inner, token, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        parse_txn_receipt(py, receipt)
    }

    fn withdraw<'py>(
        &self,
        py: Python<'py>,
        wallet: &PyEvmWallet,
        token: &str,
        amount: f64,
    ) -> PyResult<Py<PyDict>> {
        let receipt = wallet
            .rt
            .block_on(self.inner.withdraw(&wallet.inner, token, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        parse_txn_receipt(py, receipt)
    }
}
