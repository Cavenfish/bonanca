mod api_lib;
mod finance_tk;
mod utils;
mod wallets;

use crate::utils::args::{BonArgs, Bonanca};
use crate::utils::cmds::{rebalance_index_fund, show_index_balance};

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Bonanca::parse();

    match args.command {
        BonArgs::Balance(cmd) => show_index_balance(cmd).await.unwrap(),
        BonArgs::Rebalance(cmd) => rebalance_index_fund(cmd).await.unwrap(),
        BonArgs::Withdraw(cmd) => todo!(),
    };
}
