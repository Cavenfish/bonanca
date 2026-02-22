pub mod keyvault {
    pub use bonanca_keyvault::keyvault::KeyVault;
}

pub mod wallets {
    pub use bonanca_wallets::{
        HdWalletLoad, HdWalletView, WalletLoad, WalletView,
        wallets::{evm::EvmWallet, solana::SolWallet},
    };
}

#[cfg(feature = "defi")]
pub mod defi {
    pub use bonanca_defi::{
        evm::{aave::AaveV3, cow::CoW, morpho::MorphoVaultV1, zerox::ZeroX},
        solana::{jupiter::Jupiter, kamino::Kamino},
    };
}

#[cfg(feature = "oracle")]
pub mod oracle {
    pub use bonanca_oracle::prices::{CoinMarketCap, DefiLlama, DexScreener};
}
