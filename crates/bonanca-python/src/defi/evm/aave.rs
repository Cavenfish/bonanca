use bonanca_defi::evm::aave::{AaveV3, AaveV3UserData};
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::wallets::evm::PyEvmWallet;

#[pyclass(name = "AaveV3UserData")]
pub struct PyAaveV3UserData {
    pub total_collateral: f64,
    pub total_debt: f64,
    pub ltv: f64,
    pub health_factor: f64,
    pub liquidation_threshold: f64,
    pub available_borrows: f64,
}

impl PyAaveV3UserData {
    fn from_rust(data: AaveV3UserData) -> Self {
        Self {
            total_collateral: data.total_collateral,
            total_debt: data.total_debt,
            ltv: data.ltv,
            health_factor: data.health_factor,
            liquidation_threshold: data.liquidation_threshold,
            available_borrows: data.available_borrows,
        }
    }
}

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

    // fn get_pools_api(&self, chain_id: u64) -> PyResult<Vec<AaveV3ReserveData>> {
    //     let rt = Runtime::new()?;
    //     let result = rt.block_on(self.inner.get_pools_api(chain_id))?;
    //     Ok(result)
    // }

    fn get_user_data(&self, user: &str, wallet: &PyEvmWallet) -> PyResult<PyAaveV3UserData> {
        let result = wallet
            .rt
            .block_on(self.inner.get_user_data(user, &wallet.inner.client))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(PyAaveV3UserData::from_rust(result))
    }

    // fn get_reserve_data(&self, token: &str, wallet: &PyEvmWallet) -> PyResult<String> {
    //     let result = wallet
    //         .rt
    //         .block_on(self.inner.get_reserve_data(token, &wallet.inner.client))
    //         .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    //     Ok(serde_json::to_string(&result).unwrap())
    // }

    fn supply(&self, wallet: &PyEvmWallet, token: &str, amount: f64) -> PyResult<()> {
        let _ = wallet
            .rt
            .block_on(self.inner.supply(&wallet.inner, token, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()));

        Ok(())
    }

    fn borrow(&self, wallet: &PyEvmWallet, token: &str, amount: f64) -> PyResult<()> {
        let _ = wallet
            .rt
            .block_on(self.inner.borrow(&wallet.inner, token, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()));

        Ok(())
    }

    fn repay(&self, wallet: &PyEvmWallet, token: &str, amount: f64) -> PyResult<()> {
        let _ = wallet
            .rt
            .block_on(self.inner.repay(&wallet.inner, token, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()));

        Ok(())
    }

    fn withdraw(&self, wallet: &PyEvmWallet, token: &str, amount: f64) -> PyResult<()> {
        let _ = wallet
            .rt
            .block_on(self.inner.withdraw(&wallet.inner, token, amount))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()));

        Ok(())
    }
}
