use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NativeFlow {
    pub block: u64,
    pub timestamp: String,
    pub value: f64,
    pub gas_used: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenFlow {
    pub block: u64,
    pub timestamp: String,
    pub token: String,
    pub value: f64,
    pub gas_used: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CashFlow {
    pub native: Vec<NativeFlow>,
    pub tokens: Vec<TokenFlow>,
}

impl CashFlow {
    pub fn new() -> Self {
        Self {
            native: Vec::new(),
            tokens: Vec::new(),
        }
    }
}
