mod args;
mod index;
mod lend;
mod wallet;

use crate::{
    args::{BonArgs, Bonanca},
    index::cmds::handle_index_cmd,
    wallet::cmds::handle_wallet_cmd,
};

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Bonanca::parse();

    match args.command {
        BonArgs::Wallet(cmd) => handle_wallet_cmd(cmd).await,
        BonArgs::Index(cmd) => handle_index_cmd(cmd).await,
        BonArgs::Lend(cmd) => todo!(),
    }
}
