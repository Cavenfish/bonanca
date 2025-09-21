use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Bonanca {
    #[clap(subcommand)]
    pub command: BonArgs,
}

#[derive(Debug, Subcommand)]
pub enum BonArgs {
    /// List managed indexes
    List,

    /// Print the balance of indexes
    Balance(BalArgs),

    /// Rebalance indexes
    Rebalance(RebalArgs),

    /// Print summary of managed indexes
    Summary(SummaryArgs),

    /// Withdraw from index
    Withdraw(OutArgs),
}

#[derive(Debug, Args)]
pub struct BalArgs {
    /// Name of index
    #[arg(short)]
    pub index: String,
}

#[derive(Debug, Args)]
pub struct RebalArgs {
    /// Name of index
    #[arg(short)]
    pub index: String,
}

#[derive(Debug, Args)]
pub struct SummaryArgs {
    /// Name of index
    #[arg(short)]
    pub index: String,
}

#[derive(Debug, Args)]
pub struct OutArgs {
    /// Account to send funds to
    pub to: String,

    /// Amount to withdraw
    pub amount: f64,
}
