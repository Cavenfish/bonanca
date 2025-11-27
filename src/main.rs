mod args;
mod config;
mod index;
mod lend;
mod wallet;

use bonanca_core::init_config;
use clap::Parser;

use crate::{
    args::{BonArgs, Bonanca},
    config::cmds::handle_config_cmd,
    index::cmds::handle_index_cmd,
    wallet::cmds::handle_wallet_cmd,
};

#[tokio::main]
async fn main() {
    init_config();

    let args = Bonanca::parse();

    match args.command {
        BonArgs::Config(cmd) => handle_config_cmd(cmd).await,
        BonArgs::Wallet(cmd) => handle_wallet_cmd(cmd).await,
        BonArgs::Index(cmd) => handle_index_cmd(cmd).await,
        BonArgs::Lend(cmd) => todo!(),
    }
}
