use anyhow::{Ok, Result};
use bonanca_keyvault::new;
use bonanca_managers::index_fund::IndexFund;

use crate::args::CloseArgs;

use super::args::{BalArgs, CreateArgs, InOutArgs, RebalArgs};

pub fn create_keyvault(cmd: CreateArgs) -> Result<()> {
    new(&cmd.filename, &cmd.language)?;

    Ok(())
}

pub async fn close_account(cmd: CloseArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);
    let wallet = fund.get_wallet()?;

    let assets = fund.get_all_assets()?;

    for asset in assets.iter() {
        wallet
            .transfer_all_tokens(&asset.address, &cmd.send_to)
            .await?;
    }

    wallet.close(&cmd.send_to).await?;

    Ok(())
}

pub async fn show_index_balance(cmd: BalArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);

    println!("{} Balances:", &fund.name);

    let bals = fund.get_balances().await?;

    println!("Gas Balance: {}", bals.gas);
    println!("Total Balance: {:.4}\n", bals.total);

    for asset in &bals.balances {
        let actual = asset.value / bals.total;
        println!(
            "{}: {:.4} ({:.4}/{:.4})",
            asset.name, asset.value, actual, asset.target
        );
    }

    Ok(())
}

pub async fn rebalance_index_fund(cmd: RebalArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);

    let dex = fund.get_exchange()?;
    let bals = fund.get_balances().await?;
    let trades = fund.get_trades(&bals)?;

    println!("Gas Balance: {}", bals.gas);

    if cmd.preview {
        trades.iter().for_each(|t| println!("{}", t));
        return Ok(());
    }

    let wallet = fund.get_wallet()?;

    for trade in trades {
        let swap_data = dex
            .get_swap_data(&wallet, &trade.from, &trade.to, trade.amount)
            .await?;

        wallet.swap(swap_data).await?;
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

    let to = if cmd.token == "gas" {
        &fund.gas_address
    } else {
        &aux_assets
            .iter()
            .find(|x| x.symbol == cmd.token)
            .unwrap()
            .address
    };

    for asset in &bals.balances {
        let amount = usd_amount / asset.value;

        let swap_data = dex.get_swap_data(&wallet, &asset.addy, to, amount).await?;

        wallet.swap(swap_data).await?;
    }

    Ok(())
}

pub async fn deposit_into_index_fund(cmd: InOutArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);

    let dex = fund.get_exchange()?;
    let wallet = fund.get_wallet()?;
    let oracle = fund.get_oracle()?;

    let aux_assets = fund.auxiliary_assets.unwrap();

    let from = &aux_assets.iter().find(|x| x.symbol == cmd.token).unwrap();

    let bal = wallet.token_balance(&from.address).await?;
    let usd_bal = oracle.get_token_value(from, bal).await?;

    let assets: Vec<bonanca_core::holdings::Asset> =
        fund.sectors.iter().flat_map(|s| s.assets.clone()).collect();

    let amount = ((cmd.amount / usd_bal) / (assets.len() as f64)) * bal;

    for asset in assets {
        let swap_data = dex
            .get_swap_data(&wallet, &from.address, &asset.address, amount)
            .await?;

        wallet.swap(swap_data).await?;
    }
    Ok(())
}
