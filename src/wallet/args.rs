use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct WalletCommand {
    #[clap(subcommand)]
    pub command: WalletSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum WalletSubcommands {
    /// Create a new wallet
    Create(CreateArgs),
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Name of chain
    #[arg(short)]
    pub filename: PathBuf,

    /// Language for mnemonic phrase
    #[arg(long, default_value = "English")]
    pub language: String,
}
