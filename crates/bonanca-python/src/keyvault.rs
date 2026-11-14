use bonanca_keyvault::keyvault::KeyVault;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(name = "KeyVault")]
pub struct PyKeyVault {
    inner: KeyVault,
}

#[pymethods]
impl PyKeyVault {
    #[staticmethod]
    fn new(lang: String) -> PyResult<(Self, String)> {
        let (inner, mnemonic) = KeyVault::new(&lang)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok((Self { inner }, mnemonic.as_str().to_owned()))
    }

    #[staticmethod]
    fn from_mnemonic(mnemonic: String) -> PyResult<Self> {
        let inner = KeyVault::from_mnemonic(&mnemonic)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(Self { inner })
    }

    #[staticmethod]
    fn load(path: String) -> PyResult<Self> {
        let filepath = PathBuf::from(path);
        let inner = KeyVault::load(&filepath)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(Self { inner })
    }

    fn write(&self, path: String) -> PyResult<()> {
        let filepath = PathBuf::from(path);
        self.inner
            .write(&filepath)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(())
    }

    fn get_seed(&self) -> PyResult<Vec<u8>> {
        self.inner
            .get_seed()
            .map(|seed| seed.to_vec())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}
