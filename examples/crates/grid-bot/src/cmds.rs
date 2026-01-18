use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::Result;
use bonanca::{defi::ZeroX, wallets::EvmWallet};

use crate::{
    args::{BalArgs, RunArgs},
    settings::{GridBotSettings, TradePair},
};

pub async fn balance(cmds: BalArgs) -> Result<()> {
    let settings = GridBotSettings::load(&cmds.json);
    let wallet = EvmWallet::view(&settings.keyvault, &settings.rpc_url, settings.child);

    let bal_a = wallet
        .token_balance(&settings.trading_pair.token_a.address)
        .await?;
    let bal_b = wallet
        .token_balance(&settings.trading_pair.token_b.address)
        .await?;

    println!(
        "Bal {}: {:.4} || Bal {}: {:.4}",
        settings.trading_pair.token_a.symbol, bal_a, settings.trading_pair.token_b.symbol, bal_b
    );

    Ok(())
}

pub async fn run(cmds: RunArgs) -> Result<()> {
    let settings = GridBotSettings::load(&cmds.json);
    let wallet = if cmds.dry {
        EvmWallet::view(&settings.keyvault, &settings.rpc_url, settings.child)
    } else {
        EvmWallet::load(&settings.keyvault, &settings.rpc_url, settings.child)
    };
    let dex = ZeroX::new(settings.aggregator.api_key, settings.chain_id.unwrap());

    let start_time = Instant::now();
    let total_time = Duration::from_mins(cmds.mins) + Duration::from_hours(cmds.hours);

    let mut buy_levels = get_buy_levels(&settings.trading_pair)?;
    let mut sell_levels = get_sell_levels(&settings.trading_pair)?;

    while start_time.elapsed() < total_time {
        maybe_buy(
            &settings.trading_pair,
            &mut buy_levels,
            &wallet,
            &dex,
            cmds.dry,
        )
        .await?;
        maybe_sell(
            &settings.trading_pair,
            &mut sell_levels,
            &wallet,
            &dex,
            cmds.dry,
        )
        .await?;
        sleep(Duration::from_mins(cmds.interval));
    }

    Ok(())
}

async fn maybe_buy(
    pair: &TradePair,
    levels: &mut Vec<f64>,
    wallet: &EvmWallet,
    dex: &ZeroX,
    dry: bool,
) -> Result<()> {
    let quote = dex
        .get_swap_quote(
            &wallet,
            &pair.token_a.address,
            &pair.token_b.address,
            pair.buy_amount,
        )
        .await
        .unwrap();

    let out: f64 = quote.min_buy_amount.parse::<f64>().unwrap() / 1.0e2;
    let price = 1.0 / (out / pair.sell_amount);

    for level in levels.iter_mut() {
        if &price < level {
            if dry {
                println!("I would have bought {} for {}", pair.token_b.symbol, price);
            } else {
                let txn = dex.swap(&wallet, quote).await?;
                println!("Buy Txn: {}", txn.block_hash.unwrap().to_string());
            }
            *level = pair.upper_limit * 2.0;
            break;
        }
    }
    Ok(())
}

async fn maybe_sell(
    pair: &TradePair,
    levels: &mut Vec<f64>,
    wallet: &EvmWallet,
    dex: &ZeroX,
    dry: bool,
) -> Result<()> {
    let quote = dex
        .get_swap_quote(
            &wallet,
            &pair.token_b.address,
            &pair.token_a.address,
            pair.sell_amount,
        )
        .await
        .unwrap();

    let out: f64 = quote.min_buy_amount.parse::<f64>().unwrap() / 1.0e6;
    let price = out / pair.sell_amount;

    for level in levels.iter_mut() {
        if &price > level {
            if dry {
                println!("I would have sold {} for {}", pair.token_b.symbol, price);
            } else {
                let txn = dex.swap(&wallet, quote).await?;
                println!("Buy Txn: {}", txn.block_hash.unwrap().to_string());
            }
            *level = pair.lower_limit * 2.0;
            break;
        }
    }
    Ok(())
}

fn get_buy_levels(pair: &TradePair) -> Result<Vec<f64>> {
    let mid = ((pair.upper_limit) + (pair.lower_limit)) * 0.5;
    let delta = (mid - pair.lower_limit) / (pair.num_grids as f64);

    let levels: Vec<f64> = (1..=pair.num_grids)
        .map(|n| mid - ((n as f64) * delta))
        .collect();

    Ok(levels)
}

fn get_sell_levels(pair: &TradePair) -> Result<Vec<f64>> {
    let mid = ((pair.upper_limit) + (pair.lower_limit)) * 0.5;
    let delta = (pair.upper_limit - mid) / (pair.num_grids as f64);

    let levels: Vec<f64> = (1..=pair.num_grids)
        .map(|n| (n as f64) * delta + mid)
        .collect();

    Ok(levels)
}
