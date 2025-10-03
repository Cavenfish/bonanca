use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::TokenAccountsFilter::Mint;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    native_token::sol_str_to_lamports,
    pubkey::Pubkey,
    signature::Signature,
    signer::{
        Signer,
        keypair::{Keypair, read_keypair_file, write_keypair_file},
    },
    transaction::{Transaction, VersionedTransaction},
};
use solana_system_interface::{
    instruction::{create_account, transfer},
    program,
};
use std::{path::PathBuf, str::FromStr};

use crate::wallets::traits::Wallet;

const SYSTEM_ID: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
const ATOKEN_ID: Pubkey = Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const TOKEN_ID: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub struct SolWallet {
    pub key_pair: Keypair,
    pub rpc: RpcClient,
    pub pubkey: Pubkey,
}

impl Wallet for SolWallet {
    fn load(keystore: PathBuf, rpc: String) -> Self {
        let kp = read_keypair_file(keystore).unwrap();
        let rp = RpcClient::new(rpc);
        let pk = kp.pubkey();
        Self {
            key_pair: kp,
            rpc: rp,
            pubkey: pk,
        }
    }

    fn get_pubkey(&self) -> Result<String> {
        Ok(self.pubkey.to_string())
    }

    async fn balance(&self) -> Result<f64> {
        let balance = self.rpc.get_balance(&self.pubkey).await?;
        let bal = (balance as f64) / 1e9;

        Ok(bal)
    }

    async fn transfer(&self, to: &str, amount: f64) -> Result<()> {
        let to_pubkey = Pubkey::from_str_const(to);
        let lamp = sol_str_to_lamports(&amount.to_string()).unwrap();

        let info = transfer(&self.pubkey, &to_pubkey, lamp);
        let mut trans = Transaction::new_with_payer(&[info], Some(&self.pubkey));

        let blockhash = self.rpc.get_latest_blockhash().await?;
        trans.sign(&[&self.key_pair], blockhash);

        let _ = self.rpc.send_and_confirm_transaction(&trans).await.unwrap();

        Ok(())
    }

    async fn token_balance(&self, mint: &str) -> Result<f64> {
        let mint_pubkey = Pubkey::from_str_const(mint);
        let addy = self.get_token_account(&mint_pubkey).await?;

        let bal = self.rpc.get_token_account_balance(&addy).await?;
        let amount: f64 = bal.amount.parse()?;

        Ok(amount)
    }

    async fn transfer_token(&self, mint: &str, amount: f64, to: &str) -> Result<()> {
        let to_pubkey = Pubkey::from_str_const(to);
        let mint_pubkey = Pubkey::from_str_const(mint);
        let my_addy = self.get_token_account(&mint_pubkey).await?;
        let lamp = sol_str_to_lamports(&amount.to_string()).unwrap();

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
}

impl SolWallet {
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

    pub async fn create_token_account(&self, mint: &Pubkey) -> Result<()> {
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

        Ok(())
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
            data: vec![],
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
        let token = accounts.get(0).unwrap();

        // Get token account pubkey
        let addy = Pubkey::from_str_const(&token.pubkey);

        Ok(addy)
    }
}
