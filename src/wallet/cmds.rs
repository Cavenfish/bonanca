use anyhow::Result;
use bonanca_balance_sheet::db::{read_txns, write_txns};
use bonanca_core::{config::Config, transactions::CryptoOperation};
use bonanca_keyvault::{decrypt_keyvault, new, read_keyvault};
use bonanca_wallets::{get_wallet, get_wallet_view};
use std::path::PathBuf;

use crate::wallet::args::TransferArgs;

use super::args::{
    AddArgs, BalanceArgs, CreateArgs, HistoryArgs, WalletCommand, WalletSubcommands,
};

pub async fn handle_wallet_cmd(cmd: WalletCommand) {
    match cmd.command {
        WalletSubcommands::Create(cmd) => create_keyvault(cmd).unwrap(),
        WalletSubcommands::Add(cmd) => add_pubkey(cmd).unwrap(),
        WalletSubcommands::Balance(cmd) => balance(cmd).await,
        WalletSubcommands::Transfer(cmd) => transfer(cmd).await,
        WalletSubcommands::History(cmd) => history(cmd).await,
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
    let config = Config::load();

    let keyvault = match cmd.keyvault {
        Some(fname) => fname,
        None => config.keyvault,
    };

    let name = cmd.chain.split(":").last().unwrap();

    let rpc_url = &config
        .chains_info
        .iter()
        .find(|c| c.name == name)
        .unwrap()
        .rpc_url;

    let wallet = get_wallet_view(&cmd.chain, &keyvault, rpc_url, cmd.child).unwrap();

    match cmd.token {
        Some(token) => {
            let bal = wallet.token_balance(&token).await.unwrap();
            println!("Token Balance: {}", bal);
        }
        None => {
            let bal = wallet.balance().await.unwrap();
            println!("Balance: {}", bal);
        }
    }
}

async fn transfer(cmd: TransferArgs) {
    let config = Config::load();

    let keyvault = match cmd.keyvault {
        Some(fname) => fname,
        None => config.keyvault,
    };

    let name = cmd.chain.split(":").last().unwrap();

    let rpc_url = &config
        .chains_info
        .iter()
        .find(|c| c.name == name)
        .unwrap()
        .rpc_url;

    let wallet = get_wallet(&cmd.chain, &keyvault, rpc_url, cmd.child).unwrap();

    match cmd.token {
        Some(token) => {
            wallet
                .transfer_token(&token, cmd.amount, &cmd.to)
                .await
                .unwrap();
        }
        None => {
            wallet.transfer(&cmd.to, cmd.amount).await.unwrap();
        }
    }
}

async fn history(cmd: HistoryArgs) {
    let config = Config::load();

    let name = cmd.chain.split(":").last().unwrap();

    if cmd.sync {
        let rpc_url = config.get_default_chain_rpc(name);

        let keyvault = match cmd.keyvault {
            Some(fname) => fname,
            None => config.keyvault,
        };

        let wallet = get_wallet_view(&cmd.chain, &keyvault, &rpc_url, cmd.child).unwrap();

        let txns = wallet.get_history().await.unwrap();

        write_txns(&config.database, &cmd.chain, cmd.child, txns).unwrap();
    }

    let txns = read_txns(&config.database, &cmd.chain, cmd.child).unwrap();

    for (hash, txn) in txns.iter() {
        match &txn.operation {
            CryptoOperation::Transfer(ops) => {
                if ops.to == txn.pubkey.to_lowercase() {
                    println!("Inflow of {} {}", ops.amount, ops.token);
                } else {
                    println!("Outflow of {} {}", ops.amount, ops.token);
                }
            }
            _ => {}
        }
    }
}
