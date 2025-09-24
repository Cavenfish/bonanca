mod finance_tk;
mod utils;
mod wallets;

use std::path::{Path, PathBuf};

use alloy_primitives::address;
use finance_tk::indexes::load_index_fund;
use solana_sdk::signer::Signer;
use utils::config::Config;
use wallets::evm::EvmWallet;

#[tokio::main]
async fn main() {
    let ks = PathBuf::from("./test_wallet.json");
    let wallie = EvmWallet::new(ks, "https://rpc-amoy.polygon.technology/");

    let bal = wallie.balance().await.unwrap();

    println!("{}", wallie.pubkey);
    println!("{}", bal);

    let tkn = address!("0xc3Bf644bebc4dAaC868041b4fd1342C4Ae6E934e");

    let tbal = wallie.token_balance(tkn).await.unwrap();

    println!("{}", tbal);

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
