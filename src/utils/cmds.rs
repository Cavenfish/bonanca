use anyhow::{Ok, Result};

use super::args::BalArgs;

use crate::{
    api_lib::{
        cmc::CoinMarketCap,
        jupiter::Jupiter,
        traits::{Exchange, Oracle},
        zerox::ZeroX,
    },
    finance_tk::indexes::IndexFund,
    utils::args::RebalArgs,
    wallets::{evm::EvmWallet, solana::SolWallet, traits::Wallet},
};

pub async fn show_index_balance(cmd: BalArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);
    let wallet = fund.get_wallet()?;

    println!("{} Balances:", fund.name);
    println!("Public Key: {}", wallet.get_pubkey()?);
    println!("Gas Balance: {}", wallet.balance().await?);

    let bals = fund.get_balances().await?;
    let trades = fund.get_trades(&bals)?;

    println!("Total Balance: {:.4}\n", bals.total);

    for asset in &bals.balances {
        let actual = asset.value / bals.total;
        println!(
            "{}: {:.4} ({:.4}/{:.4})",
            asset.name, asset.value, actual, asset.target
        );
    }

    println!();

    for trade in trades {
        println!("Trade {} {} for {}", trade.amount, trade.from, trade.to);
    }

    Ok(())
}

pub async fn rebalance_index_fund(cmd: RebalArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);

    let dex = fund.get_exchange()?;

    let wallet = fund.get_wallet()?;

    println!("Public Key: {}", wallet.get_pubkey()?);
    println!("Gas Balance: {}", wallet.balance().await?);

    let bals = fund.get_balances().await?;
    let trades = fund.get_trades(&bals)?;

    for trade in trades {
        let swap_data = dex
            .get_swap_data(&wallet, &trade.from, &trade.to, trade.amount)
            .await?;
        let _ = wallet.swap(swap_data).await?;
    }

    Ok(())
}
