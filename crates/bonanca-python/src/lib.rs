mod defi;
mod wallets;

use pyo3::prelude::*;

#[pymodule]
mod bonanca {

    #[pymodule_export]
    use crate::wallets::{
        evm::{PyEvmWallet, PyEvmWalletView},
        solana::{PySolWallet, PySolWalletView},
    };

    #[pymodule_export]
    use crate::defi::{evm::zerox::PyZeroX, solana::jupiter::PyJupiter};
}
