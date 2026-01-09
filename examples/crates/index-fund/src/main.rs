mod args;
mod cmds;
mod index_fund;
mod rebal_methods;

use clap::Parser;

use crate::args::{Index, IndexArgs};

#[tokio::main]
async fn main() {
    let args = Index::parse();

    match args.command {
        IndexArgs::Balance(cmd) => todo!(),
        IndexArgs::Close(cmd) => todo!(),
        IndexArgs::Deposit(cmd) => todo!(),
        IndexArgs::Rebalance(cmd) => todo!(),
        IndexArgs::Withdraw(cmd) => todo!(),
    }
}
