mod api_lib;
mod finance_tk;
mod utils;
mod wallets;

use crate::utils::args::{BonArgs, Bonanca};
use crate::utils::cmds::{
    deposit_into_index_fund, rebalance_index_fund, show_index_balance, withdraw_from_index_fund,
};

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Bonanca::parse();

    match args.command {
        BonArgs::Balance(cmd) => show_index_balance(cmd).await.unwrap(),
        BonArgs::Rebalance(cmd) => rebalance_index_fund(cmd).await.unwrap(),
        BonArgs::Withdraw(cmd) => withdraw_from_index_fund(cmd).await.unwrap(),
        BonArgs::Deposit(cmd) => deposit_into_index_fund(cmd).await.unwrap(),
    };
}
