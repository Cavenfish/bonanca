use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Index {
    #[clap(subcommand)]
    pub command: IndexArgs,
}

#[derive(Debug, Subcommand)]
pub enum IndexArgs {
    /// Print the balance of an index
    Balance(BalArgs),

    /// Rebalance index
    Rebalance(RebalArgs),
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
    #[arg(short, long, default_value = "redistribute")]
    pub method: String,

    /// Auxiliary token
    #[arg(long)]
    pub aux_token: Option<String>,

    /// Preview rebalance trades
    #[arg(short, long, action)]
    pub preview: bool,
}
