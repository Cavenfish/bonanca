use crate::{exchanges::traits::Dex, wallets::traits::Wallet};

use anyhow::Result;

pub struct ZeroX {
    pub api_key: String,
}
