use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct GridBot {
    #[clap(subcommand)]
    pub command: GridBotArgs,
}

#[derive(Debug, Subcommand)]
pub enum GridBotArgs {
    /// Print the balances
    Balance(BalArgs),

    /// Run the grid bot
    Run(RunArgs),
}

#[derive(Debug, Args)]
pub struct BalArgs {
    /// Grid bot json file
    #[arg(short)]
    pub json: PathBuf,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    /// Grid bot json file
    #[arg(short)]
    pub json: PathBuf,

    /// Order lifetime in mins (combinations possible)
    #[arg(long, default_value_t = 0)]
    pub mins: u64,

    /// Order lifetime in hours (combinations possible)
    #[arg(long, default_value_t = 2)]
    pub hours: u64,

    /// Places buy-side trades only
    #[arg(long, action)]
    pub buy_only: bool,

    /// Places sell-side trades only
    #[arg(long, action)]
    pub sell_only: bool,

    /// Dry run (prints what trades would be placed)
    #[arg(long, action)]
    pub dry: bool,
}
