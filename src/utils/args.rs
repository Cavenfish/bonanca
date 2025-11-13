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
pub struct BalArgs {
    /// Name of index
    #[arg(short)]
    pub index: PathBuf,
}

#[derive(Debug, Args)]
pub struct RebalArgs {
    /// Name of index
    #[arg(short)]
    pub index: PathBuf,
}

#[derive(Debug, Args)]
pub struct InOutArgs {
    /// Name of index
    #[arg(short)]
    pub index: PathBuf,

    /// Account to send funds to
    #[arg(short)]
    pub to: String,

    /// Amount to withdraw
    #[arg(short)]
    pub amount: f64,
}
