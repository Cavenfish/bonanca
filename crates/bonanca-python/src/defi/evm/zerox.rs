use bonanca_api_lib::defi::zerox::{Issues as RustIssues, ZeroXSwapQuote as RustZeroXSwapQuote};
use bonanca_defi::evm::zerox::ZeroX;
use pyo3::prelude::*;
use pyo3::{exceptions::PyRuntimeError, types::PyDict};

use crate::wallets::evm::{PyEvmWallet, parse_txn_receipt};

#[pyclass(name = "ZeroX")]
pub struct PyZeroX {
    inner: ZeroX,
}

#[pymethods]
impl PyZeroX {
    #[new]
    fn new(api_key: String, chain_id: u16) -> Self {
        let inner = ZeroX::new(api_key, chain_id);
        Self { inner }
    }

    fn check_swap<'py>(
        &self,
        py: Python<'py>,
        wallet: &PyEvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> PyResult<Py<PyDict>> {
        let issues = wallet
            .rt
            .block_on(self.inner.check_swap(&wallet.inner, sell, buy, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        let dict = PyDict::new(py);

        dict.set_item("simulation_incomplete", issues.simulation_incomplete)?;
        if let Some(allowance) = issues.allowance {
            dict.set_item("allowance_issue", format!("spender: {}", allowance.spender))?;
        }
        if let Some(balance) = issues.balance {
            dict.set_item("balance_issue", format!("token: {}", balance.token))?;
        }

        Ok(dict.into())
    }

    fn get_swap_quote(
        &self,
        wallet: &PyEvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> PyResult<PyZeroXSwapQuote> {
        let quote = wallet
            .rt
            .block_on(self.inner.get_swap_quote(&wallet.inner, sell, buy, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(PyZeroXSwapQuote::from_rust(quote))
    }

    fn swap<'py>(
        &self,
        py: Python<'py>,
        wallet: &PyEvmWallet,
        quote: &PyZeroXSwapQuote,
    ) -> PyResult<Py<PyDict>> {
        let rust_quote = quote.clone().to_rust();

        let receipt = wallet
            .rt
            .block_on(self.inner.swap(&wallet.inner, rust_quote))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        parse_txn_receipt(py, receipt)
    }

    fn quick_swap<'py>(
        &self,
        py: Python<'py>,
        wallet: &PyEvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> PyResult<Py<PyDict>> {
        let receipt = wallet
            .rt
            .block_on(self.inner.quick_swap(&wallet.inner, sell, buy, amount))
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        parse_txn_receipt(py, receipt)
    }
}

#[pyclass(name = "ZeroXSwapQuote")]
#[derive(Clone)]
pub struct PyZeroXSwapQuote {
    pub buy_amount: String,
    pub buy_token: String,
    pub sell_amount: String,
    pub sell_token: String,
    pub min_buy_amount: String,
    pub allowance_target: String,
    pub liquidity_available: bool,
}

impl PyZeroXSwapQuote {
    fn from_rust(quote: RustZeroXSwapQuote) -> Self {
        Self {
            buy_amount: quote.buy_amount,
            buy_token: quote.buy_token,
            sell_amount: quote.sell_amount,
            sell_token: quote.sell_token,
            min_buy_amount: quote.min_buy_amount,
            allowance_target: quote.allowance_target,
            liquidity_available: quote.liquidity_available,
        }
    }

    fn to_rust(self) -> RustZeroXSwapQuote {
        RustZeroXSwapQuote {
            allowance_target: self.allowance_target,
            block_number: String::new(),
            buy_amount: self.buy_amount,
            buy_token: self.buy_token,
            fees: bonanca_api_lib::defi::zerox::Fees {
                integrator_fee: None,
                zero_ex_fee: None,
                gas_fee: None,
            },
            issues: RustIssues {
                allowance: None,
                balance: None,
                simulation_incomplete: false,
                invalid_sources_passed: vec![],
            },
            liquidity_available: self.liquidity_available,
            min_buy_amount: self.min_buy_amount,
            route: bonanca_api_lib::defi::zerox::Route {
                fills: vec![],
                tokens: vec![],
            },
            sell_amount: self.sell_amount,
            sell_token: self.sell_token,
            token_metadata: bonanca_api_lib::defi::zerox::TokenMetadata {
                buy_token: bonanca_api_lib::defi::zerox::TokenTax {
                    buy_tax_bps: String::new(),
                    sell_tax_bps: String::new(),
                },
                sell_token: bonanca_api_lib::defi::zerox::TokenTax {
                    buy_tax_bps: String::new(),
                    sell_tax_bps: String::new(),
                },
            },
            total_network_fee: String::new(),
            transaction: bonanca_api_lib::defi::zerox::Transaction {
                data: String::new(),
                gas: String::new(),
                gas_price: String::new(),
                to: String::new(),
                value: String::new(),
            },
            zid: String::new(),
        }
    }
}
