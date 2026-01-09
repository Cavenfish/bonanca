pub mod wallets {
    pub use bonanca_wallets::wallets::{evm::EvmWallet, solana::SolWallet};
}

pub mod defi {
    pub use bonanca_defi::{
        evm::{aave::AaveV3, zerox::ZeroX},
        solana::jupiter::Jupiter,
    };
}

pub mod oracle {
    pub use bonanca_oracle::prices::{CoinMarketCap, DefiLlama};
}
