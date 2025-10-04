mod cmc_api;
mod exchanges;
mod finance_tk;
mod utils;
mod wallets;

use std::path::PathBuf;
use wallets::evm::EvmWallet;
use wallets::solana::SolWallet;

use cmc_api::get::get_token_value;
use solana_sdk::pubkey::Pubkey;
use utils::config::Config;

use crate::utils::args::{BonArgs, Bonanca};
use crate::utils::cmds::show_index_balance;
use crate::wallets::traits::Wallet;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Bonanca::parse();

    match args.command {
        BonArgs::Balance(cmd) => show_index_balance(cmd).await.unwrap(),
        BonArgs::Rebalance(cmd) => todo!(),
        BonArgs::Withdraw(cmd) => todo!(),
    };

    // let cfg = Config::load_account().unwrap();
    // let ks = dirs::data_dir().unwrap().join("bonanca/keypair.json");
    // let rpc = "https://api.devnet.solana.com".to_string();
    // let wallie = SolWallet::from(ks, rpc);

    // let sell = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    // let buy = Pubkey::from_str("3wQct2e43J1Z99h2RWrhPAhf6E32ZpuzEt6tgwfEAKAy").unwrap();

    // let _ = wallie.create_token_account(&buy).await.unwrap();
    // println!("{}", id());

    // let _ = wallie.close_token_account(&buy).await.unwrap();

    // let tkn = wallie.get_token_account(buy).await.unwrap();
    // let bal = wallie.token_balance(buy).await.unwrap();

    // println!("{}", tkn);
    // println!("{}", bal);

    // let _ = wallie.swap(&sell, &buy, 1_000_000).await.unwrap();

    // let value = get_token_value(1, 1.0, &cfg.api_url, &cfg.api_key)
    //     .await
    //     .unwrap();

    // println!("{:?}", value);

    // let ks = PathBuf::from("./test_wallet.json");
    // let wallie = EvmWallet::load(ks, "https://rpc-amoy.polygon.technology/".to_string());

    // let bal = wallie.balance().await.unwrap();

    // println!("{}", wallie.pubkey);
    // println!("{}", bal);

    // let tkn = address!("0xc3Bf644bebc4dAaC868041b4fd1342C4Ae6E934e");

    // let tbal = wallie.token_balance(tkn).await.unwrap();

    // println!("{}", tbal);

    // let cfg = Config::load_account().unwrap();
    // let kp = cfg.get_keypair().unwrap();
    // let rpc = cfg.get_rpc_client().unwrap();

    // let wallie = SolWallet::new(kp, rpc);

    // println!("Wallet pubkey: {}", wallie.key_pair.pubkey());
    // println!("Amount: {}", wallie.balance().await);

    // let fname = Path::new("./fund.json");

    // let fund = load_index_fund(fname).unwrap();

    // println!("{:?}", fund);

    // println!("");
    // println!("");

    // let accts = wallie.get_accounts().await.unwrap();

    // println!("{:?}", accts);
}
