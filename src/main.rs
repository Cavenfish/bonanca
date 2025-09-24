mod finance_tk;
mod utils;
mod wallets;

use std::path::Path;

use finance_tk::indexes::load_index_fund;
use solana_sdk::signer::Signer;
use utils::config::Config;
use wallets::solana::SolWallet;

#[tokio::main]
async fn main() {
    let cfg = Config::load_account().unwrap();
    let kp = cfg.get_keypair().unwrap();
    let rpc = cfg.get_rpc_client().unwrap();

    let wallie = SolWallet::new(kp, rpc);

    println!("Wallet pubkey: {}", wallie.key_pair.pubkey());
    println!("Amount: {}", wallie.balance().await);

    let fname = Path::new("./fund.json");

    let fund = load_index_fund(fname).unwrap();

    println!("{:?}", fund);

    println!("");
    println!("");

    let accts = wallie.get_accounts().await.unwrap();

    println!("{:?}", accts);
}
