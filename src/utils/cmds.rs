use super::args::BalArgs;

use crate::{
    finance_tk::indexes::load_index_fund,
    wallets::{evm::EvmWallet, solana::SolWallet, traits::Wallet},
};

use anyhow::{Ok, Result};

pub async fn show_index_balance(cmd: BalArgs) -> Result<()> {
    let fund = load_index_fund(&cmd.index)?;

    let wallet: Box<dyn Wallet> = match fund.chain.as_str() {
        "EVM" => Box::new(EvmWallet::load(fund.keystore, fund.rpc_url)),
        "Solana" => Box::new(SolWallet::load(fund.keystore, fund.rpc_url)),
        _ => Err(anyhow::anyhow!("Unsupported chain"))?,
    };

    println!("{} Balances:", fund.name);
    println!("Public Key: {}", wallet.get_pubkey()?);
    println!("Gas Balance: {}", wallet.balance().await?);

    for sector in fund.sectors {
        println!("{} Sector ({})", sector.name, sector.weight);
        for asset in sector.assets {
            let bal = wallet.token_balance(&asset.token).await?;

            println!("{}: {}", asset.name, bal);
        }
    }

    Ok(())
}
