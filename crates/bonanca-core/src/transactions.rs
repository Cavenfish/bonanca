use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct Txn {
    pub pubkey: String,
    pub block: u64,
    pub timestamp: u64,
    pub gas_used: f64,
    pub operation: CryptoOperation,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum CryptoOperation {
    Approve(EvmApprove),
    Trade(CryptoTrade),
    Transfer(CryptoTransfer),
    Banking(CryptoBanking),
    Liquidity(CryptoLP),
    None,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct EvmApprove {
    pub token: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CryptoTrade {
    pub sell: String,
    pub buy: String,
    pub sell_amount: f64,
    pub buy_amount: f64,
    // pub sell_value: f64,
    // pub buy_value: f64,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CryptoTransfer {
    pub token: String,
    pub amount: f64,
    // pub value: f64,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CryptoBanking {
    pub token: String,
    pub amount: f64,
    pub action: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CryptoLP {
    pub token_a: String,
    pub token_b: String,
    pub amount_a: f64,
    pub amount_b: f64,
    // pub total_value: f64,
}
