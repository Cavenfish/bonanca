mod defi;
mod wallets;

use pyo3::prelude::*;

#[pymodule]
mod bonanca {
    #[pymodule_export]
    use super::pywallets;

    #[pymodule_export]
    use super::pydefi;
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
