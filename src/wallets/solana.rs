use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use solana_system_interface::instruction::transfer;

pub struct SolWallet {
    pub key_pair: Keypair,
    pub rpc: RpcClient,
}

impl SolWallet {
    pub fn new(key_pair: Keypair, rpc: RpcClient) -> Self {
        Self { key_pair, rpc }
    }

    pub async fn balance(&self) -> f64 {
        let balance = self.rpc.get_balance(&self.key_pair.pubkey()).await.unwrap();
        (balance as f64) / 1e9
    }

    pub async fn transfer(&self, to: &Pubkey, amount: u64) -> Result<Signature> {
        let payer = self.key_pair.pubkey();
        let info = transfer(&payer, to, amount);
        let mut trans = Transaction::new_with_payer(&[info], Some(&payer));
        let blockhash = self.rpc.get_latest_blockhash().await?;

        trans.sign(&[&self.key_pair], blockhash);

        let sig = self.rpc.send_and_confirm_transaction(&trans).await.unwrap();

        Ok(sig)
    }
}
