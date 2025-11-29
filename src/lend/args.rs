use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct LendCommand {
    #[clap(subcommand)]
    pub command: LendSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum LendSubcommands {
    Show(ShowArgs),

    Balance(BalArgs),
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    #[arg(short)]
    pub json_file: PathBuf,
}

#[derive(Debug, Args)]
pub struct BalArgs {
    #[arg(short)]
    pub json_file: PathBuf,
}
