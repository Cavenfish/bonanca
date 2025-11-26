use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct LendCommand {
    #[clap(subcommand)]
    pub command: LendSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum LendSubcommands {}
