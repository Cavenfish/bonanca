mod finance_tk;
mod utils;
mod wallets;

use solana_sdk::signer::Signer;
use utils::config::load_account;
use wallets::solana::SolWallet;

#[tokio::main]
async fn main() {
    let (target, rpc) = load_account().unwrap();

    let wallie = SolWallet::new(target, rpc);

    println!("Wallet pubkey: {}", wallie.key_pair.pubkey());
    println!("Amount: {}", wallie.balance().await);
}
