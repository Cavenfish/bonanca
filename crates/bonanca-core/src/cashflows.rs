use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NativeFlow {
    pub block: u64,
    pub timestamp: String,
    pub to: String,
    pub from: String,
    pub value: f64,
    pub gas_used: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenFlow {
    pub block: u64,
    pub timestamp: String,
    pub token: String,
    pub to: String,
    pub from: String,
    pub value: f64,
    pub gas_used: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CashFlow {
    pub pubkey: String,
    pub native: Vec<NativeFlow>,
    pub tokens: Vec<TokenFlow>,
}

impl CashFlow {
    pub fn new() -> Self {
        Self {
            pubkey: String::new(),
            native: Vec::new(),
            tokens: Vec::new(),
        }
    }
}
