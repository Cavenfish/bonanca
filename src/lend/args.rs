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
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    #[arg(short)]
    pub json_file: PathBuf,
}
