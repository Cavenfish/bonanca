use crate::{
    exchanges::traits::{Dex, SwapData},
    wallets::traits::Wallet,
};

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::deserialize;
use jup_ag_sdk::{
    JupiterClient,
    types::{QuoteGetSwapModeEnum, QuoteRequest, SwapRequest, SwapResponse},
};
use solana_sdk::transaction::Transaction;

pub struct Jup {
    pub client: JupiterClient,
}

impl Jup {
    pub fn new() -> Self {
        let client = JupiterClient::new("https://lite-api.jup.ag");
        Self { client: client }
    }
}

impl Dex for Jup {
    async fn get_swap_data(
        &self,
        wallet: &Box<dyn Wallet>,
        sell: &str,
        buy: &str,
        amount: u64,
    ) -> Result<SwapData> {
        let pubkey = wallet.get_pubkey()?;

        let quote = QuoteRequest::new(&sell.to_string(), &buy.to_string(), amount)
            .swap_mode(QuoteGetSwapModeEnum::ExactOut);

        let quote_res = self.client.get_quote(&quote).await?;

        let payload = SwapRequest::new(&pubkey, &pubkey, quote_res).wrap_and_unwrap_sol(true);

        println!("{:?}", &payload);

        let swap_res: SwapResponse = self.client.get_swap_transaction(&payload).await?;

        println!("{:?}", &swap_res);

        let swap_tx_bytes = STANDARD.decode(swap_res.swap_transaction)?;

        let trans: Transaction = deserialize(&swap_tx_bytes).unwrap();

        println!("Check 2");

        Ok(SwapData::Sol(trans))
    }
}
