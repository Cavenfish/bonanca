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

    /// Add public key to keyvault json
    Add(AddArgs),

    /// Get token balance
    Balance(BalanceArgs),

    /// Transfer token
    Transfer(TransferArgs),
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Name of keyvault file
    #[arg(short)]
    pub filename: PathBuf,

    /// Language for mnemonic phrase
    #[arg(long, default_value = "English")]
    pub language: String,
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Keyvault file
    #[arg(long)]
    pub keyvault: Option<PathBuf>,

    /// Chain to add pubkey
    #[arg(short)]
    pub chain: String,

    /// Child index for pubkey
    #[arg(short = 'i')]
    pub child: u32,
}

#[derive(Debug, Args)]
pub struct BalanceArgs {
    /// Keyvault file
    #[arg(short)]
    pub keyvault: PathBuf,

    /// Token address
    #[arg(short)]
    pub token: String,
}

#[derive(Debug, Args)]
pub struct TransferArgs {
    /// Keyvault file
    #[arg(short)]
    pub keyvault: PathBuf,

    /// Token address
    #[arg(short)]
    pub token: String,

    /// Amount
    #[arg(short)]
    pub amount: f64,
}
