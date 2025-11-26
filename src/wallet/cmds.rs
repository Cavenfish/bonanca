use anyhow::Result;
use bonanca_core::get_wallet_view;
use bonanca_keyvault::new;

use crate::wallet::{self, args::BalanceArgs};

use super::args::{CreateArgs, WalletCommand, WalletSubcommands};

pub async fn handle_wallet_cmd(cmd: WalletCommand) {
    match cmd.command {
        WalletSubcommands::Create(cmd) => create_keyvault(cmd).unwrap(),
        WalletSubcommands::Balance(cmd) => todo!(),
        WalletSubcommands::Transfer(cmd) => todo!(),
    };
}

fn create_keyvault(cmd: CreateArgs) -> Result<()> {
    new(&cmd.filename, &cmd.language)?;

    Ok(())
}

async fn balance(cmd: BalanceArgs) {
    // let wallet = get_wallet_view(chain, keyvault, rpc_url, child)?;
}
