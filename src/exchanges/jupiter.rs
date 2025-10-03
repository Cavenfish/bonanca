use crate::{exchanges::traits::Dex, wallets::traits::Wallet};

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
    async fn swap<T: Wallet>(&self, wallet: T, sell: &str, buy: &str, amount: u64) -> Result<()> {
        let pubkey = wallet.get_pubkey()?;

        let quote = QuoteRequest::new(&sell.to_string(), &buy.to_string(), amount)
            .swap_mode(QuoteGetSwapModeEnum::ExactOut);

        let quote_res = self.client.get_quote(&quote).await?;

        let payload = SwapRequest::new(&pubkey, &pubkey, quote_res);

        println!("Check 0");

        let swap_res: SwapResponse = self.client.get_swap_transaction(&payload).await?;

        println!("Check 1");

        let swap_tx_bytes = STANDARD.decode(swap_res.swap_transaction)?;

        let mut trans: Transaction = deserialize(&swap_tx_bytes).unwrap();

        println!("Check 2");

        // Get latest blockhash and sign transaction
        // let blockhash = wallet.rpc.get_latest_blockhash().await?;
        // trans.sign(&[&wallet.key_pair], blockhash);

        // // Send and wait for confirmation
        // let _ = wallet
        //     .rpc
        //     .send_and_confirm_transaction(&trans)
        //     .await
        //     .unwrap();

        Ok(())
    }
}
