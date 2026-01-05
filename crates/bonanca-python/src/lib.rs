use pyo3::prelude::*;

#[pymodule]
mod bonanca {
    use std::path::PathBuf;

    use bonanca_wallets::wallets::{evm::EvmWallet, solana::SolWallet};
    use pyo3::prelude::*;

    #[pyclass]
    pub struct PyEvmWallet {
        inner: EvmWallet,
    }

    #[pymethods]
    impl PyEvmWallet {
        #[new]
        fn new(keyvault: PathBuf, rpc: &str, child: u32) -> Self {
            let inner = EvmWallet::view(&keyvault, rpc, child);
            Self { inner }
        }

        fn get_pubkey(&self) -> String {
            self.inner.get_pubkey().unwrap()
        }

        async fn balance(&self) -> f64 {
            self.inner.balance().await.unwrap()
        }
    }

    #[pyclass]
    pub struct PySolWallet {
        inner: SolWallet,
    }

    #[pymethods]
    impl PySolWallet {
        #[new]
        fn new(keyvault: PathBuf, rpc: &str, child: u32) -> Self {
            let inner = SolWallet::view(&keyvault, rpc, child);
            Self { inner }
        }

        fn get_pubkey(&self) -> String {
            self.inner.get_pubkey().unwrap()
        }

        async fn balance(&self) -> f64 {
            self.inner.balance().await.unwrap()
        }
    }
}
