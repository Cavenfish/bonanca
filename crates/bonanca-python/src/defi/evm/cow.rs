use bonanca_api_lib::defi::cow::CowSwapPlacedOrder;
use bonanca_defi::evm::cow::CoW;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use std::time::Duration;

use crate::wallets::evm::PyEvmWallet;

#[pyclass(name = "CoW")]
pub struct PyCoW {
    inner: CoW,
}

#[pymethods]
impl PyCoW {
    #[new]
    fn new(chain: &str) -> PyResult<Self> {
        let inner = CoW::new(chain)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { inner })
    }

    fn get_order_info<'py>(&self, py: Python<'py>, uid: &str) -> PyResult<Py<PyDict>> {
        let rt = tokio::runtime::Runtime::new()?;
        let info = rt.block_on(self.inner.get_order_info(uid)).unwrap();
        let dict = cow_swap_placed_order_to_dict(py, &info)?;

        Ok(dict.into())
    }

    fn get_user_orders<'py>(
        &self,
        py: Python<'py>,
        user: &str,
        limit: u16,
    ) -> PyResult<Py<PyList>> {
        let rt = tokio::runtime::Runtime::new()?;
        let info = rt
            .block_on(self.inner.get_user_orders(user, Some(limit)))
            .unwrap();

        let vec_dict: Vec<Bound<'_, PyDict>> = info
            .iter()
            .map(|i| cow_swap_placed_order_to_dict(py, i).unwrap())
            .collect();

        let list = PyList::new(py, vec_dict)?;

        Ok(list.into())
    }

    fn market_order(
        &self,
        wallet: &PyEvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
    ) -> PyResult<String> {
        let quote = wallet
            .rt
            .block_on(
                self.inner
                    .get_market_quote(&wallet.inner, sell, buy, amount),
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        wallet
            .rt
            .block_on(self.inner.post_market_order(&wallet.inner, quote))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn limit_order(
        &self,
        wallet: &PyEvmWallet,
        sell: &str,
        buy: &str,
        sell_amount: f64,
        buy_amount: f64,
        lifetime: (u64, u64),
    ) -> PyResult<String> {
        let lifetime = Duration::from_hours(lifetime.0) + Duration::from_mins(lifetime.1);
        wallet
            .rt
            .block_on(self.inner.limit_order(
                &wallet.inner,
                sell,
                buy,
                sell_amount,
                buy_amount,
                lifetime,
            ))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn limit_order_by_price(
        &self,
        wallet: &PyEvmWallet,
        sell: &str,
        buy: &str,
        amount: f64,
        sell_price: f64,
        lifetime: (u64, u64),
    ) -> PyResult<String> {
        let lifetime = Duration::from_hours(lifetime.0) + Duration::from_mins(lifetime.1);
        wallet
            .rt
            .block_on(self.inner.limit_order_by_price(
                &wallet.inner,
                sell,
                buy,
                amount,
                sell_price,
                lifetime,
            ))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

fn cow_swap_placed_order_to_dict<'py>(
    py: Python<'py>,
    order: &CowSwapPlacedOrder,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);

    dict.set_item("creation_date", &order.creation_date)?;
    dict.set_item("owner", &order.owner)?;
    dict.set_item("uid", &order.uid)?;
    dict.set_item("available_balance", &order.available_balance)?;
    dict.set_item("executed_buy_amount", &order.executed_buy_amount)?;
    dict.set_item("executed_sell_amount", &order.executed_sell_amount)?;
    dict.set_item(
        "executed_sell_amount_before_fees",
        &order.executed_sell_amount_before_fees,
    )?;
    dict.set_item("executed_fee_amount", &order.executed_fee_amount)?;
    dict.set_item("executed_fee", &order.executed_fee)?;
    dict.set_item("executed_fee_token", &order.executed_fee_token)?;
    dict.set_item("invalidated", order.invalidated)?;
    dict.set_item("status", &order.status)?;
    dict.set_item("class", &order.class)?;
    dict.set_item("settlement_contract", &order.settlement_contract)?;
    dict.set_item("is_liquidity_order", order.is_liquidity_order)?;
    dict.set_item("full_app_data", &order.full_app_data)?;
    dict.set_item("sell_token", &order.sell_token)?;
    dict.set_item("buy_token", &order.buy_token)?;
    dict.set_item("receiver", &order.receiver)?;
    dict.set_item("sell_amount", &order.sell_amount)?;
    dict.set_item("buy_amount", &order.buy_amount)?;
    dict.set_item("valid_to", order.valid_to)?;
    dict.set_item("app_data", &order.app_data)?;
    dict.set_item("fee_amount", &order.fee_amount)?;
    dict.set_item("kind", &order.kind)?;
    dict.set_item("partially_fillable", order.partially_fillable)?;
    dict.set_item("sell_token_balance", &order.sell_token_balance)?;
    dict.set_item("buy_token_balance", &order.buy_token_balance)?;
    dict.set_item("signing_scheme", &order.signing_scheme)?;
    dict.set_item("signature", &order.signature)?;

    // Handle quote field
    if let Some(quote) = &order.quote {
        let quote_dict = PyDict::new(py);
        quote_dict.set_item("gas_amount", &quote.gas_amount)?;
        quote_dict.set_item("gas_price", &quote.gas_price)?;
        quote_dict.set_item("sell_token_price", &quote.sell_token_price)?;
        quote_dict.set_item("sell_amount", &quote.sell_amount)?;
        quote_dict.set_item("buy_amount", &quote.buy_amount)?;
        quote_dict.set_item("fee_amount", &quote.fee_amount)?;
        quote_dict.set_item("solver", &quote.solver)?;
        quote_dict.set_item("verified", quote.verified)?;
        dict.set_item("quote", quote_dict)?;
    } else {
        dict.set_item("quote", py.None())?;
    }

    // Handle interactions field
    let interactions_dict = PyDict::new(py);
    interactions_dict.set_item("pre", &order.interactions.pre)?;
    interactions_dict.set_item("post", &order.interactions.post)?;
    dict.set_item("interactions", interactions_dict)?;

    Ok(dict)
}
