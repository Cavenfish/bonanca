use super::args::BalArgs;

use crate::{
    exchanges::{jupiter::Jup, traits::Dex},
    finance_tk::indexes::load_index_fund,
    utils::args::RebalArgs,
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

pub async fn rebalance_index_fund(cmd: RebalArgs) -> Result<()> {
    let fund = load_index_fund(&cmd.index)?;

    let wallet: Box<dyn Wallet> = match fund.chain.as_str() {
        "EVM" => Box::new(EvmWallet::load(fund.keystore, fund.rpc_url)),
        "Solana" => Box::new(SolWallet::load(fund.keystore, fund.rpc_url)),
        _ => Err(anyhow::anyhow!("Unsupported chain"))?,
    };

    println!("Public Key: {}", wallet.get_pubkey()?);
    println!("Gas Balance: {}", wallet.balance().await?);

    let dex = Jup::new();

    let sell = "So11111111111111111111111111111111111111112";
    let buy = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
    let amount = 1_000;

    let swap_data = dex.get_swap_data(&wallet, sell, buy, amount).await?;

    let _ = wallet.swap(swap_data).await?;

    Ok(())
}
