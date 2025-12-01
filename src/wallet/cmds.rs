use anyhow::Result;
use bonanca_core::config::Config;
use bonanca_keyvault::{decrypt_keyvault, new, read_keyvault};
use std::path::PathBuf;

use super::args::{AddArgs, BalanceArgs, CreateArgs, WalletCommand, WalletSubcommands};

pub async fn handle_wallet_cmd(cmd: WalletCommand) {
    match cmd.command {
        WalletSubcommands::Create(cmd) => create_keyvault(cmd).unwrap(),
        WalletSubcommands::Add(cmd) => add_pubkey(cmd).unwrap(),
        WalletSubcommands::Balance(cmd) => todo!(),
        WalletSubcommands::Transfer(cmd) => todo!(),
    };
}

fn get_default_keyvault() -> PathBuf {
    let config = Config::load();

    config.keyvault
}

fn create_keyvault(cmd: CreateArgs) -> Result<()> {
    new(&cmd.filename, &cmd.language)?;

    Ok(())
}

fn add_pubkey(cmd: AddArgs) -> Result<()> {
    let fname = match cmd.keyvault {
        Some(fname) => fname,
        None => get_default_keyvault(),
    };

    let hd_key = decrypt_keyvault(&fname)?;
    let child_pubkey = hd_key.get_child_pubkey(&cmd.chain, cmd.child)?;

    let mut keyvault = read_keyvault(&fname)?;

    keyvault
        .chain_keys
        .iter_mut()
        .find(|k| k.chain == cmd.chain)
        .unwrap()
        .public_keys
        .push(child_pubkey);

    keyvault.write(&fname);

    Ok(())
}

async fn balance(cmd: BalanceArgs) {
    // let wallet = get_wallet_view(chain, keyvault, rpc_url, child)?;
}
