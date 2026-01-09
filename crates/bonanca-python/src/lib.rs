mod defi;
mod oracle;
mod wallets;

use pyo3::prelude::*;

#[pymodule]
mod bonanca {
    #[pymodule_export]
    use super::pywallets;

    #[pymodule_export]
    use super::pydefi;

    #[pymodule_export]
    use super::pyoracle;
}

#[pymodule(name = "wallets", submodule)]
mod pywallets {
    #[pymodule_export]
    use crate::wallets::{
        evm::{PyEvmWallet, PyEvmWalletView},
        solana::{PySolWallet, PySolWalletView},
    };
}

#[pymodule(name = "defi", submodule)]
mod pydefi {
    #[pymodule_export]
    use crate::defi::{evm::zerox::PyZeroX, solana::jupiter::PyJupiter};
}

#[pymodule(name = "oracle", submodule)]
mod pyoracle {
    #[pymodule_export]
    use crate::oracle::prices::{PyCoinMarketCap, PyDefiLlama};
}
