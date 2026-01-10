use bonanca_keyvault::keyvault::KeyVault;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[pyclass(name = "KeyVault")]
pub struct PyKeyVault {
    inner: KeyVault,
}

#[pymethods]
impl PyKeyVault {
    #[staticmethod]
    fn new(lang: String) -> PyResult<Self> {
        let inner = KeyVault::new(&lang);
        Ok(Self { inner })
    }

    #[staticmethod]
    fn from_mnemonic(mnemonic: String) -> PyResult<Self> {
        let inner = KeyVault::from_mnemonic(&mnemonic);
        Ok(Self { inner })
    }

    #[staticmethod]
    fn load(path: String) -> PyResult<Self> {
        let filepath = PathBuf::from(path);
        let inner = KeyVault::load(&filepath);
        Ok(Self { inner })
    }

    fn write(&self, path: String) -> PyResult<()> {
        let filepath = PathBuf::from(path);
        self.inner.write(&filepath);
        Ok(())
    }

    fn get_seed(&self) -> PyResult<Vec<u8>> {
        self.inner
            .get_seed()
            .map(|seed| seed.to_vec())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn make_new_child(&mut self, chain: String) -> PyResult<()> {
        self.inner
            .make_new_child(&chain)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn make_new_children(&mut self, chain: String, num_kids: u32) -> PyResult<()> {
        self.inner
            .make_new_children(&chain, num_kids)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn chain_keys(&self) -> PyResult<HashMap<String, Vec<String>>> {
        Ok(self.inner.chain_keys.clone())
    }
}
