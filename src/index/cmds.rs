use anyhow::{Ok, Result};
use bonanca_db::BonancaDB;
use bonanca_managers::index_fund::IndexFund;

use super::args::{BalArgs, CloseArgs, InOutArgs, IndexCommand, IndexSubcommands, RebalArgs};

pub async fn handle_index_cmd(cmd: IndexCommand) {
    match cmd.command {
        IndexSubcommands::Close(cmd) => close_account(cmd).await.unwrap(),
        IndexSubcommands::Balance(cmd) => show_index_balance(cmd).await.unwrap(),
        IndexSubcommands::Rebalance(cmd) => rebalance_index_fund(cmd).await.unwrap(),
        IndexSubcommands::Withdraw(cmd) => withdraw_from_index_fund(cmd).await.unwrap(),
        IndexSubcommands::Deposit(cmd) => deposit_into_index_fund(cmd).await.unwrap(),
    };
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

    println!("{: <15} {: <10} {: <12}", "Token", "Balance", "Allocation");
    println!("{:-<40}", "");

    for asset in &bals.balances {
        let actual = asset.value / bals.total;
        println!(
            "{: <15} {:<10.4} ({:<5.4}/{:<5.4})",
            asset.name, asset.value, actual, asset.target
        );
    }

    println!("\nAuxiliary Assets");

    for asset in &bals.aux_balances {
        println!("{: <15} {:<10.4}", asset.name, asset.value);
    }

    Ok(())
}

pub async fn rebalance_index_fund(cmd: RebalArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);

    let dex = fund.get_exchange()?;
    let bals = fund.get_balances().await?;
    let aux_token_symbol = cmd.aux_token.unwrap_or("".to_string());

    let aux_token = if aux_token_symbol.is_empty() {
        ""
    } else {
        let aux_assets = fund.auxiliary_assets.as_ref().unwrap();
        &aux_assets
            .iter()
            .find(|a| a.symbol == aux_token_symbol)
            .unwrap()
            .address
            .clone()
    };

    if aux_token.is_empty() && cmd.method != "redistribute" {
        panic!()
    }

    let trades = fund.get_trades(&bals, &cmd.method, aux_token)?;

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

        wallet.sign_and_send(swap_data).await?;
    }

    Ok(())
}

pub async fn withdraw_from_index_fund(cmd: InOutArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);
    let db = BonancaDB::load();

    let dex = fund.get_exchange()?;
    let wallet = fund.get_wallet()?;

    let bals = fund.get_balances().await?;

    let usd_amount = cmd.amount / (bals.balances.len() as f64);

    let aux_assets = fund.auxiliary_assets.unwrap();

    let to = if cmd.token == "gas" {
        &db.read_chain_info(&fund.chain)?.wrapped_native
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

        wallet.sign_and_send(swap_data).await?;
    }

    Ok(())
}

pub async fn deposit_into_index_fund(cmd: InOutArgs) -> Result<()> {
    let fund = IndexFund::load(&cmd.index);

    let dex = fund.get_exchange()?;
    let wallet = fund.get_wallet()?;

    let aux_assets = fund.auxiliary_assets.unwrap();

    let from = &aux_assets.iter().find(|x| x.symbol == cmd.token).unwrap();

    for sector in fund.sectors.iter() {
        let target = sector.weight / (sector.assets.len() as f64);

        for asset in sector.assets.iter() {
            let amount = cmd.amount * target;
            let swap_data = dex
                .get_swap_data(&wallet, &from.address, &asset.address, amount)
                .await?;

            wallet.sign_and_send(swap_data).await?;
        }
    }

    Ok(())
}
