use anyhow::Result;

use crate::index_fund::{IndexBalances, RebalTrade};

pub fn make_rebal_trades(bals: &IndexBalances, max_offset: f64) -> Result<Vec<RebalTrade>> {
    let mut addys: Vec<String> = Vec::new();
    let mut diffs: Vec<f64> = Vec::new();
    let mut amounts: Vec<f64> = Vec::new();
    let mut actuals: Vec<f64> = Vec::new();

    for asset in &bals.balances {
        let bal = asset.value;
        let actual = bal / bals.total;
        let diff = asset.target - actual;

        addys.push(asset.addy.clone());
        diffs.push(diff);
        amounts.push(asset.amount);
        actuals.push(actual);
    }

    let mut order = (0..diffs.len()).collect::<Vec<_>>();
    order.sort_by_key(|&k| (&diffs[k] * 1e6) as i64);

    let n = diffs.len();
    let mut trades: Vec<RebalTrade> = Vec::new();

    for i in 0..(n - 1) {
        let small = order[i];

        let mut j = n - 1;
        while diffs[small].abs() > max_offset {
            let big = order[j];

            if diffs[big] < 0.0 {
                println!("Two negative numbers");
                break;
            }

            let diff = if diffs[big].abs() > diffs[small].abs() {
                diffs[small].abs()
            } else {
                diffs[big].abs()
            };

            if diff == 0.0 {
                j -= 1;
                continue;
            }

            let frac = diff / actuals[small];
            let amount = frac * amounts[small];

            trades.push(RebalTrade {
                from: addys[small].clone(),
                to: addys[big].clone(),
                amount,
            });

            diffs[small] += diff;
            diffs[big] -= diff;
            j -= 1;
        }
    }

    Ok(trades)
}

pub fn make_skim_trades(
    bals: &IndexBalances,
    to: &str,
    max_offset: f64,
) -> Result<Vec<RebalTrade>> {
    let mut trades: Vec<RebalTrade> = Vec::new();

    for asset in &bals.balances {
        let actual = asset.value / bals.total;

        if actual > asset.target {
            let amount = (actual - asset.target) * asset.amount;
            trades.push(RebalTrade {
                from: asset.addy.clone(),
                to: to.to_string(),
                amount,
            });
        }
    }

    Ok(trades)
}

pub fn make_buyin_trades(
    bals: &IndexBalances,
    from: &str,
    usd_per_from_token: f64,
    max_offset: f64,
) -> Result<Vec<RebalTrade>> {
    let mut trades: Vec<RebalTrade> = Vec::new();

    for asset in &bals.balances {
        let actual = asset.value / bals.total;

        if actual < asset.target {
            let frac = asset.target - actual;
            let usd_amount = frac * bals.total;
            let amount = usd_amount / usd_per_from_token;

            trades.push(RebalTrade {
                from: asset.addy.clone(),
                to: asset.addy.clone(),
                amount,
            });
        }
    }

    Ok(trades)
}
