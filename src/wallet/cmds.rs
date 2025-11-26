use anyhow::Result;
use bonanca_keyvault::new;

use super::args::{CreateArgs, WalletCommand, WalletSubcommands};

pub async fn handle_wallet_cmd(cmd: WalletCommand) {
    match cmd.command {
        WalletSubcommands::Create(cmd) => create_keyvault(cmd).unwrap(),
    };
}

pub fn create_keyvault(cmd: CreateArgs) -> Result<()> {
    new(&cmd.filename, &cmd.language)?;

    Ok(())
}
