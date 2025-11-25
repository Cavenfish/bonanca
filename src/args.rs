use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Bonanca {
    #[clap(subcommand)]
    pub command: BonArgs,
}

#[derive(Debug, Subcommand)]
pub enum BonArgs {
    /// Create a new wallet
    Create(CreateArgs),

    /// Close a wallet
    Close(CloseArgs),

    /// Print the balance of an index
    Balance(BalArgs),

    /// Rebalance index
    Rebalance(RebalArgs),

    /// Withdraw from index
    Withdraw(InOutArgs),

    /// Deposit into index
    Deposit(InOutArgs),
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

#[derive(Debug, Args)]
pub struct CloseArgs {
    /// Index fund json file
    #[arg(short)]
    pub index: PathBuf,

    /// Wallet to send funds to
    #[arg(short)]
    pub send_to: String,
}

#[derive(Debug, Args)]
pub struct BalArgs {
    /// Index fund json file
    #[arg(short)]
    pub index: PathBuf,
}

#[derive(Debug, Args)]
pub struct RebalArgs {
    /// Index fund json file
    #[arg(short)]
    pub index: PathBuf,

    /// Method for rebalancing
    #[arg(short, long)]
    pub method: String,

    /// Auxiliary token
    #[arg(long)]
    pub aux_token: Option<String>,

    /// Preview rebalance trades
    #[arg(short, long, action)]
    pub preview: bool,
}

#[derive(Debug, Args)]
pub struct InOutArgs {
    /// Index fund json file
    #[arg(short)]
    pub index: PathBuf,

    /// Auxiliary token
    #[arg(short)]
    pub token: String,

    /// Amount to withdraw/deposit
    #[arg(short)]
    pub amount: f64,
}
