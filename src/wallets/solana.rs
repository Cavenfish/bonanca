use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    account::Account,
    entrypoint::__Pubkey,
    pubkey::Pubkey,
    signature::Signature,
    signer::{
        Signer,
        keypair::{Keypair, read_keypair_file, write_keypair_file},
    },
    transaction::Transaction,
};
use solana_system_interface::instruction::transfer;
use std::path::PathBuf;

pub struct SolWallet {
    pub key_pair: Keypair,
    pub rpc: RpcClient,
    pub pubkey: __Pubkey,
}

impl SolWallet {
    pub fn new(keystore: PathBuf, rpc: String) -> Self {
        let kp = read_keypair_file(keystore).unwrap();
        let rp = RpcClient::new(rpc);
        let pk = kp.pubkey();
        Self {
            key_pair: kp,
            rpc: rp,
            pubkey: pk,
        }
    }

    pub async fn balance(&self) -> Result<f64> {
        let balance = self.rpc.get_balance(&self.pubkey).await?;
        let bal = (balance as f64) / 1e9;

        Ok(bal)
    }

    pub async fn transfer(&self, to: &Pubkey, amount: u64) -> Result<Signature> {
        // Build transfer instructions
        let info = transfer(&self.pubkey, to, amount);
        let mut trans = Transaction::new_with_payer(&[info], Some(&self.pubkey));

        // Get latest blockhash and sign transaction
        let blockhash = self.rpc.get_latest_blockhash().await?;
        trans.sign(&[&self.key_pair], blockhash);

        // Send and wait for confirmation
        let sig = self.rpc.send_and_confirm_transaction(&trans).await.unwrap();

        Ok(sig)
    }

    pub async fn get_accounts(&self) -> Result<Vec<(Pubkey, Account)>> {
        let accts = self.rpc.get_program_accounts(&self.pubkey).await?;

        Ok(accts)
    }
}
