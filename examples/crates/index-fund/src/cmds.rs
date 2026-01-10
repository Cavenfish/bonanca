use anyhow::Result;
use bonanca::defi::{Jupiter, ZeroX};
use bonanca::wallets::{EvmWallet, SolWallet};

use crate::args::{BalArgs, RebalArgs};
use crate::index_fund::{IndexFund, RebalTrade};

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

    let _ = make_trades(&fund, trades).await?;

    Ok(())
}

async fn make_trades(fund: &IndexFund, trades: Vec<RebalTrade>) -> Result<()> {
    let chain = if fund.chain.contains(":") {
        fund.chain.split(":").next().unwrap()
    } else {
        &fund.chain
    };

    match (chain, fund.aggregator.name.as_str()) {
        ("EVM", "0x") => {
            let wallet = EvmWallet::load(&fund.keyvault, &fund.rpc_url, fund.child);
            let dex = ZeroX::new(fund.aggregator.api_key.clone(), fund.chain_id.unwrap());

            for trade in trades.iter() {
                let issues = dex
                    .check_swap(&wallet, &trade.from, &trade.to, trade.amount)
                    .await
                    .unwrap();

                if let Some(allowance) = issues.allowance {
                    let amount = trade.amount - allowance.actual.parse::<f64>().unwrap();
                    wallet
                        .approve_token_spending(&trade.from, &allowance.spender, amount)
                        .await
                        .unwrap();
                }

                let _ = dex
                    .swap(&wallet, &trade.from, &trade.to, trade.amount)
                    .await
                    .unwrap();
            }
        }
        ("Solana", "Jupiter") => {
            let wallet = SolWallet::load(&fund.keyvault, &fund.rpc_url, fund.child);
            let dex = Jupiter::new(fund.aggregator.api_key.clone());

            for trade in trades.iter() {
                let _ = dex
                    .swap(&wallet, &trade.from, &trade.to, trade.amount)
                    .await
                    .unwrap();
            }
        }
        _ => panic!(),
    }

    Ok(())
}
