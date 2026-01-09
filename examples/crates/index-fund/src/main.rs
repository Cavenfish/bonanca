mod args;
mod cmds;
mod index_fund;
mod rebal_methods;

use clap::Parser;

use args::{Index, IndexArgs};
use cmds::show_index_balance;

use crate::cmds::rebalance_index_fund;

#[tokio::main]
async fn main() {
    let args = Index::parse();

    match args.command {
        IndexArgs::Balance(cmd) => show_index_balance(cmd).await.unwrap(),
        IndexArgs::Rebalance(cmd) => rebalance_index_fund(cmd).await.unwrap(),
    }
}
