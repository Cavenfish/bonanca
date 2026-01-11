pub mod keyvault {
    pub use bonanca_keyvault::keyvault::KeyVault;
}

pub mod wallets {
    pub use bonanca_wallets::wallets::{evm::EvmWallet, solana::SolWallet};
}

pub mod defi {
    pub use bonanca_defi::{
        evm::{aave::AaveV3, morpho::MorphoVaultV1, zerox::ZeroX},
        solana::jupiter::Jupiter,
    };
}

pub mod oracle {
    pub use bonanca_oracle::prices::{CoinMarketCap, DefiLlama};
}
