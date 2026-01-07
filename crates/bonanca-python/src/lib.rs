mod wallets;

use pyo3::prelude::*;

#[pymodule]
mod bonanca {

    #[pymodule_export]
    use crate::wallets::{
        evm::{PyEvmWallet, PyEvmWalletView},
        solana::{PySolWallet, PySolWalletView},
    };
}
