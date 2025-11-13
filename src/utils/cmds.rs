use anyhow::{Ok, Result};

use super::args::BalArgs;

use crate::{
    finance_tk::indexes::IndexFund,
    utils::args::{InOutArgs, RebalArgs},
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

pub async fn withdraw_from_index_fund(cmd: InOutArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);

    let dex = fund.get_exchange()?;
    let wallet = fund.get_wallet()?;

    let bals = fund.get_balances().await?;

    let usd_amount = cmd.amount / (bals.balances.len() as f64);

    let aux_assets = fund.auxiliary_assets.unwrap();

    let to = &aux_assets
        .iter()
        .find(|x| x.symbol == cmd.to)
        .unwrap()
        .address;

    for asset in &bals.balances {
        let amount = usd_amount / asset.value;

        let swap_data = dex.get_swap_data(&wallet, &asset.addy, &to, amount).await?;
        let _ = wallet.swap(swap_data).await?;
    }

    Ok(())
}

pub async fn deposit_into_index_fund(cmd: InOutArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);

    let dex = fund.get_exchange()?;
    let wallet = fund.get_wallet()?;
    let oracle = fund.get_oracle()?;

    let aux_assets = fund.auxiliary_assets.unwrap();

    let from = &aux_assets.iter().find(|x| x.symbol == cmd.to).unwrap();

    let bal = wallet.token_balance(&from.address).await?;
    let usd_bal = oracle.get_token_value(from, bal).await?;

    let assets: Vec<crate::finance_tk::indexes::Asset> =
        fund.sectors.iter().flat_map(|s| s.assets.clone()).collect();

    let amount = ((cmd.amount / usd_bal) / (assets.len() as f64)) * bal;

    for asset in assets {
        let swap_data = dex
            .get_swap_data(&wallet, &from.address, &asset.address, amount)
            .await?;
        let _ = wallet.swap(swap_data).await?;
    }
    Ok(())
}
