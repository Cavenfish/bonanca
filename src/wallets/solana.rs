use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::deserialize;
use jup_ag_sdk::{
    JupiterClient,
    types::{QuoteGetSwapModeEnum, QuoteRequest, SwapRequest, SwapResponse},
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::TokenAccountsFilter::Mint;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
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

const SYSTEM_ID: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
const ATOKEN_ID: Pubkey = Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const TOKEN_ID: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

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

    pub async fn token_balance(&self, mint: &Pubkey) -> Result<String> {
        // Get token account pubkey
        let addy = self.get_token_account(mint).await?;

        // Get token balance
        let bal = self.rpc.get_token_account_balance(&addy).await?;

        Ok(bal.amount)
    }

    pub async fn transfer_token(&self, mint: &Pubkey, amount: u64, to: &Pubkey) -> Result<()> {
        // Get token account pubkey
        let my_addy = self.get_token_account(mint).await?;

        // Get to token account pubkey
        let accounts = self
            .rpc
            .get_token_accounts_by_owner(to, Mint(*mint))
            .await?;
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

    pub async fn swap(&self, sell: &Pubkey, buy: &Pubkey, amount: u64) -> Result<()> {
        let client = JupiterClient::new("https://lite-api.jup.ag");

        let quote = QuoteRequest::new(&sell.to_string(), &buy.to_string(), amount)
            .swap_mode(QuoteGetSwapModeEnum::ExactOut);

        let quote_res = client.get_quote(&quote).await?;

        let payload = SwapRequest::new(
            &self.pubkey.to_string(),
            &self.pubkey.to_string(),
            quote_res,
        );

        println!("Check 0");

        let swap_res: SwapResponse = client.get_swap_transaction(&payload).await?;

        println!("Check 1");

        let swap_tx_bytes = STANDARD.decode(swap_res.swap_transaction)?;

        let mut trans: Transaction = deserialize(&swap_tx_bytes).unwrap();

        println!("Check 2");

        // Get latest blockhash and sign transaction
        let blockhash = self.rpc.get_latest_blockhash().await?;
        trans.sign(&[&self.key_pair], blockhash);

        // Send and wait for confirmation
        let _ = self.rpc.send_and_confirm_transaction(&trans).await.unwrap();

        Ok(())
    }
}
