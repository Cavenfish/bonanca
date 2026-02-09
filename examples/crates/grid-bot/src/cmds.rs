use std::time::Duration;

use anyhow::{Result, bail};
use bonanca::{defi::CoW, wallets::EvmWallet};

use crate::{
    args::{BalArgs, RunArgs},
    db::GridBotLog,
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
    if cmds.sell_only && cmds.buy_only {
        bail!("Buy_only and sell_only cannot both be true");
    }

    let settings = GridBotSettings::load(&cmds.json);
    let wallet = if cmds.dry {
        EvmWallet::view(&settings.keyvault, &settings.rpc_url, settings.child)
    } else {
        EvmWallet::load(&settings.keyvault, &settings.rpc_url, settings.child)
    };
    let cow = CoW::new(&settings.chain)?;

    let mut log = GridBotLog::load(&settings);
    log.prune_log(&settings.chain).await?;

    let lifetime = Duration::from_hours(cmds.hours) + Duration::from_mins(cmds.mins);

    if !cmds.sell_only {
        let buy_levels = get_buy_levels(&settings.trading_pair)?;
        for level in buy_levels.into_iter() {
            if cmds.dry {
                println!("Buy level at: {}", level);
            } else {
                let uid = cow
                    .limit_order_by_price(
                        &wallet,
                        &settings.trading_pair.token_a.address,
                        &settings.trading_pair.token_b.address,
                        settings.trading_pair.sell_amount,
                        level,
                        lifetime,
                    )
                    .await?;
                log.active_orders.push(uid);
            }
        }
    }

    if !cmds.buy_only {
        let sell_levels = get_sell_levels(&settings.trading_pair)?;
        for level in sell_levels.into_iter() {
            if cmds.dry {
                println!("Sell level at: {}", level);
            } else {
                let uid = cow
                    .limit_order_by_price(
                        &wallet,
                        &settings.trading_pair.token_b.address,
                        &settings.trading_pair.token_a.address,
                        settings.trading_pair.sell_amount,
                        1.0 / level,
                        lifetime,
                    )
                    .await?;
                log.active_orders.push(uid);
            }
        }
    }

    log.write(&settings.log_file)?;

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
