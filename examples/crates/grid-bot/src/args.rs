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

    /// Time in minutes between trade queries
    #[arg(long, default_value_t = 5)]
    pub interval: u64,

    /// Time in minutes to run bot
    #[arg(long, default_value_t = 5)]
    pub mins: u64,

    /// Time in hours to run bot
    #[arg(long, default_value_t = 0)]
    pub hours: u64,

    /// Dry run (prints what trades would be done)
    #[arg(long, action)]
    pub dry: bool,
}
