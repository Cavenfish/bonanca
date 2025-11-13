use anyhow::{Context, Result};
use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::TokenAccountsFilter::Mint;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signer::{
        Signer,
        keypair::{Keypair, read_keypair_file},
    },
    transaction::Transaction,
};
use solana_system_interface::instruction::transfer;
use std::path::Path;

use crate::api_lib::traits::SwapTransactionData;
use crate::wallets::traits::Wallet;

const SYSTEM_ID: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
const ATOKEN_ID: Pubkey = Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const TOKEN_ID: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub struct SolWallet {
    pub key_pair: Keypair,
    pub rpc: RpcClient,
    pub pubkey: Pubkey,
}

#[async_trait]
impl Wallet for SolWallet {
    fn get_pubkey(&self) -> Result<String> {
        Ok(self.pubkey.to_string())
    }

    fn parse_native_amount(&self, amount: f64) -> Result<u64> {
        let amt = (amount * 1e9) as u64;

        Ok(amt)
    }

    async fn parse_token_amount(&self, amount: f64, token: &str) -> Result<u64> {
        let pubkey = Pubkey::from_str_const(token);
        let token_addy = self.get_token_account(&pubkey).await?;
        let acct = self.rpc.get_token_account(&token_addy).await?.unwrap();

        let deci = acct.token_amount.decimals;

        let amt = (amount * 10.0_f64.powi(deci.into())) as u64;

        Ok(amt)
    }

    async fn balance(&self) -> Result<f64> {
        let balance = self.rpc.get_balance(&self.pubkey).await?;
        let bal = (balance as f64) / 1e9;

        Ok(bal)
    }

    async fn transfer(&self, to: &str, amount: f64) -> Result<()> {
        let to_pubkey = Pubkey::from_str_const(to);
        let lamp = self.parse_native_amount(amount)?;

        let info = transfer(&self.pubkey, &to_pubkey, lamp);
        let mut trans = Transaction::new_with_payer(&[info], Some(&self.pubkey));

        let blockhash = self.rpc.get_latest_blockhash().await?;
        trans.sign(&[&self.key_pair], blockhash);

        let _ = self.rpc.send_and_confirm_transaction(&trans).await.unwrap();

        Ok(())
    }

    async fn token_balance(&self, mint: &str) -> Result<f64> {
        let mint_pubkey = Pubkey::from_str_const(mint);
        let addy_result = self.get_token_account(&mint_pubkey).await;

        let addy = match addy_result {
            Ok(addy) => addy,
            Err(_) => self.create_token_account(&mint_pubkey).await?,
        };

        let token_data = self.rpc.get_token_account_balance(&addy).await?;
        let bal = token_data.ui_amount.unwrap_or(0.0);

        Ok(bal)
    }

    async fn transfer_token(&self, mint: &str, amount: f64, to: &str) -> Result<()> {
        let to_pubkey = Pubkey::from_str_const(to);
        let mint_pubkey = Pubkey::from_str_const(mint);
        let my_addy = self.get_token_account(&mint_pubkey).await?;
        let lamp = self.parse_token_amount(amount, mint).await?;

        let accounts = self
            .rpc
            .get_token_accounts_by_owner(&to_pubkey, Mint(mint_pubkey))
            .await?;

        let token = accounts.get(0).unwrap();
        let token_pubkey = Pubkey::from_str_const(&token.pubkey);

        let info = transfer(&my_addy, &token_pubkey, lamp);
        let mut trans = Transaction::new_with_payer(&[info], Some(&self.pubkey));

        let blockhash = self.rpc.get_latest_blockhash().await?;
        trans.sign(&[&self.key_pair], blockhash);

        let _ = self.rpc.send_and_confirm_transaction(&trans).await.unwrap();

        Ok(())
    }

    async fn check_swap(&self, token: &str, amount: f64, _spender: Option<&str>) -> Result<bool> {
        let bal = self.token_balance(token).await?;

        if bal < amount {
            return Ok(false);
        };

        Ok(true)
    }

    async fn swap(&self, swap_data: SwapTransactionData) -> Result<()> {
        let mut tx = match swap_data {
            SwapTransactionData::Sol(trans) => trans,
            _ => Err(anyhow::anyhow!("Swap API does not work on this chain"))?,
        };

        let message = tx.message.serialize();
        let signature = self.key_pair.sign_message(&message);

        if tx.signatures.is_empty() {
            // If no signatures array exists (unlikely with Jupiter)
            tx.signatures.push(signature);
        } else {
            // Replace the first signature (fee payer)
            tx.signatures[0] = signature;
        };

        let _ = self.rpc.send_and_confirm_transaction(&tx).await?;

        Ok(())
    }
}

impl SolWallet {
    pub fn load(keystore: &Path, rpc: &str) -> Self {
        let kp = read_keypair_file(keystore).unwrap();
        let rp = RpcClient::new(rpc.to_string());
        let pk = kp.pubkey();
        Self {
            key_pair: kp,
            rpc: rp,
            pubkey: pk,
        }
    }

    pub async fn build_sign_and_send(&self, instr: Instruction) -> Result<()> {
        // Get blockhash and sign transaction
        let blockhash = self.rpc.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[instr],
            Some(&self.pubkey),
            &[&self.key_pair],
            blockhash,
        );

        // Send and wait for confirmation
        let _ = self.rpc.send_and_confirm_transaction(&tx).await?;

        Ok(())
    }

    pub async fn create_token_account(&self, mint: &Pubkey) -> Result<Pubkey> {
        // Get associated token account address
        let (token_account, _) = Pubkey::find_program_address(
            &[
                &self.pubkey.to_bytes(),
                &TOKEN_ID.to_bytes(),
                &mint.to_bytes(),
            ],
            &ATOKEN_ID,
        );

        // Build create instructions
        let instr = Instruction {
            program_id: ATOKEN_ID,
            accounts: vec![
                AccountMeta::new(self.pubkey, true),
                AccountMeta::new(token_account, false),
                AccountMeta::new_readonly(self.pubkey, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new_readonly(SYSTEM_ID, false),
                AccountMeta::new_readonly(TOKEN_ID, false),
            ],
            data: vec![0],
        };

        // Get blockhash and sign transaction
        let _ = self.build_sign_and_send(instr).await?;

        Ok(token_account)
    }

    pub async fn close_token_account(&self, mint: &Pubkey) -> Result<()> {
        let token_account = self.get_token_account(mint).await?;

        // Build close instructions
        let instr = Instruction {
            program_id: TOKEN_ID,
            accounts: vec![
                AccountMeta::new(token_account, false),
                AccountMeta::new(self.pubkey, true),
                AccountMeta::new(self.pubkey, true),
            ],
            data: vec![9],
        };

        // Get blockhash and sign transaction
        let _ = self.build_sign_and_send(instr).await?;

        Ok(())
    }

    pub async fn get_token_account(&self, mint: &Pubkey) -> Result<Pubkey> {
        // Get token account
        let accounts = self
            .rpc
            .get_token_accounts_by_owner(&self.pubkey, Mint(*mint))
            .await?;
        let token = accounts.get(0).context("Could not find token account")?;

        // Get token account pubkey
        let addy = Pubkey::from_str_const(&token.pubkey);

        Ok(addy)
    }
}
