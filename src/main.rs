mod cmc_api;
mod finance_tk;
mod utils;
mod wallets;

use cmc_api::get::get_token_value;
use utils::config::Config;

#[tokio::main]
async fn main() {
    let cfg = Config::load_account().unwrap();

    let value = get_token_value(1, 1.0, &cfg.api_url, &cfg.api_key)
        .await
        .unwrap();

    println!("{:?}", value);

    // let ks = PathBuf::from("./test_wallet.json");
    // let wallie = EvmWallet::new(ks, "https://rpc-amoy.polygon.technology/");

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
