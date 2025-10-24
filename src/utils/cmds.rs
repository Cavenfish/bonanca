use anyhow::{Ok, Result};

use super::args::BalArgs;

use crate::{
    api_lib::{
        cmc::CoinMarketCap,
        jupiter::Jupiter,
        traits::{Exchange, Oracle},
        zerox::ZeroX,
    },
    finance_tk::indexes::load_index_fund,
    utils::args::RebalArgs,
    wallets::{evm::EvmWallet, solana::SolWallet, traits::Wallet},
};

pub async fn show_index_balance(cmd: BalArgs) -> Result<()> {
    let fund = load_index_fund(&cmd.index)?;

    let wallet: Box<dyn Wallet> = match fund.chain.as_str() {
        "EVM" => Box::new(EvmWallet::load(fund.keystore, fund.rpc_url)),
        "Solana" => Box::new(SolWallet::load(fund.keystore, fund.rpc_url)),
        _ => Err(anyhow::anyhow!("Unsupported chain"))?,
    };

    let oracle = CoinMarketCap::new(fund.oracle.api_url, fund.oracle.api_key);

    println!("{} Balances:", fund.name);
    println!("Public Key: {}", wallet.get_pubkey()?);
    println!("Gas Balance: {}", wallet.balance().await?);

    for sector in fund.sectors {
        println!("{} Sector ({})", sector.name, sector.weight);
        for asset in sector.assets {
            let bal = wallet.token_balance(&asset.token).await?;

            let usd = if bal != 0.0 {
                oracle.get_token_value(&asset.name, bal).await?
            } else {
                0.0
            };

            println!("{}: {}", asset.name, usd);
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

    // let dex = Jupiter::new(fund.aggregator.api_url, fund.aggregator.api_key);
    let dex = ZeroX::new(
        fund.aggregator.api_url,
        fund.aggregator.api_key,
        fund.chain_id.unwrap(),
    );

    // let sell = "So11111111111111111111111111111111111111112";
    // let buy = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
    let sell = "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619";
    let buy = "0xb33EaAd8d922B1083446DC23f610c2567fB5180f";
    let amount = 0.000006;

    // let swap_data = dex.get_ultra_order(sell, buy, amount, &taker).await?;
    let swap_data = dex.get_swap_data(&wallet, sell, buy, amount).await?;

    let _ = wallet.swap(swap_data).await?;

    Ok(())
}
