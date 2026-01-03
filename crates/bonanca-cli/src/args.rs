use clap::{Parser, Subcommand};

use crate::{
    config::args::ConfigCommand, index::args::IndexCommand, lend::args::LendCommand,
    wallet::args::WalletCommand,
};

#[derive(Debug, Parser)]
#[command(version, about, author)]
pub struct Bonanca {
    #[clap(subcommand)]
    pub command: BonArgs,
}

#[derive(Debug, Subcommand)]
pub enum BonArgs {
    /// Manage your config file
    Config(ConfigCommand),

    /// Interact with your wallets
    Wallet(WalletCommand),

    /// Manage index fund
    Index(IndexCommand),

    /// Manage loan portfolio
    Lend(LendCommand),
}
