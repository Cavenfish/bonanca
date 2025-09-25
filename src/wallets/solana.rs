use alloy::sol_types::sol_data::Address;
use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::TokenAccountsFilter::Mint;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::Signature,
    signer::{
        Signer,
        keypair::{Keypair, read_keypair_file, write_keypair_file},
    },
    transaction::Transaction,
};
use solana_system_interface::{
    instruction::{create_account, transfer},
    program,
};
use std::path::PathBuf;

pub struct SolWallet {
    pub key_pair: Keypair,
    pub rpc: RpcClient,
    pub pubkey: Pubkey,
}

impl SolWallet {
    pub fn from(keystore: PathBuf, rpc: String) -> Self {
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

    pub async fn create_token_account(&self, mint: &Pubkey) -> Result<()> {
        // Get rent
        let rent = self.rpc.get_minimum_balance_for_rent_exemption(165).await?;

        // Make keypair for token account
        let new_kp = Keypair::new();

        // Build instructions and get blockhash
        let instr = create_account(&self.pubkey, &new_kp.pubkey(), rent, 165, &program::id());
        let blockhash = self.rpc.get_latest_blockhash().await?;

        // Sign transaction
        let tx = Transaction::new_signed_with_payer(
            &[instr],
            Some(&self.pubkey),
            &[&self.key_pair, &new_kp],
            blockhash,
        );

        // Send and wait for confirmation
        let _ = self.rpc.send_and_confirm_transaction(&tx).await?;

        Ok(())
    }

    pub async fn get_token_account(&self, mint: Pubkey) -> Result<Pubkey> {
        // Get token account
        let accounts = self
            .rpc
            .get_token_accounts_by_owner(&self.pubkey, Mint(mint))
            .await?;
        let token = accounts.get(0).unwrap();

        // Get token account pubkey
        let addy = Pubkey::from_str_const(&token.pubkey);

        Ok(addy)
    }

    pub async fn token_balance(&self, mint: Pubkey) -> Result<String> {
        // Get token account pubkey
        let addy = self.get_token_account(mint).await?;

        // Get token balance
        let bal = self.rpc.get_token_account_balance(&addy).await?;

        Ok(bal.amount)
    }

    pub async fn transfer_token(&self, mint: Pubkey, amount: u64, to: &Pubkey) -> Result<()> {
        // Get token account pubkey
        let my_addy = self.get_token_account(mint).await?;

        // Get to token account pubkey
        let accounts = self.rpc.get_token_accounts_by_owner(to, Mint(mint)).await?;
        let token = accounts.get(0).unwrap();
        let to_addy = Pubkey::from_str_const(&token.pubkey);

        // Build transfer
        let info = transfer(&my_addy, &to_addy, amount);
        let mut trans = Transaction::new_with_payer(&[info], Some(&self.pubkey));

        // Get latest blockhash and sign transaction
        let blockhash = self.rpc.get_latest_blockhash().await?;
        trans.sign(&[&self.key_pair], blockhash);

        // Send and wait for confirmation
        let _ = self.rpc.send_and_confirm_transaction(&trans).await.unwrap();

        Ok(())
    }
}
