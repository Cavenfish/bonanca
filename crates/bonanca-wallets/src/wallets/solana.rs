use anyhow::{Context, Result};
use async_trait::async_trait;
use bonanca_db::{
    BonancaDB,
    transactions::{CryptoOperation, CryptoTransfer, Txn},
};
use bonanca_keyvault::{decrypt_keyvault, hd_keys::ChildKey, read_keyvault};
use solana_client::rpc_request::TokenAccountsFilter::Mint;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::UiTransactionEncoding};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    signer::{Signer, keypair::Keypair},
    transaction::{Transaction, VersionedTransaction},
};
use solana_system_interface::instruction::transfer;
use std::{hash::Hash, path::Path, str::FromStr};

const SYSTEM_ID: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
const ATOKEN_ID: Pubkey = Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const TOKEN_ID: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub struct SolWallet {
    pub key_pair: Option<Keypair>,
    pub client: RpcClient,
    pub pubkey: Pubkey,
}

impl SolWallet {
    pub fn load(keyvault: &Path, rpc: &str, child: u32) -> Self {
        let hd_key = decrypt_keyvault(keyvault).expect("Failed to decrypt keyvault");
        let child_key = hd_key.get_child_key("Solana", child).unwrap();

        let kp = match child_key {
            ChildKey::Sol(kp) => kp,
            _ => panic!(),
        };

        let client = RpcClient::new(rpc.to_string());
        let pubkey = kp.pubkey();
        Self {
            key_pair: Some(kp),
            client,
            pubkey,
        }
    }

    pub fn view(keyvault: &Path, rpc: &str, child: u32) -> Self {
        let key_vault = read_keyvault(keyvault).unwrap();
        let sol_keys = key_vault
            .chain_keys
            .iter()
            .find(|k| k.chain == "Solana")
            .unwrap();
        let pubkey = sol_keys.public_keys.get(child as usize).unwrap();
        let client = RpcClient::new(rpc.to_string());
        let pubkey = Pubkey::from_str_const(pubkey);
        Self {
            key_pair: None,
            client,
            pubkey,
        }
    }

    async fn build_sign_and_send(&self, instr: Instruction) -> Result<()> {
        let kp = self.key_pair.as_ref().unwrap();

        // Get blockhash and sign transaction
        let blockhash = self.client.get_latest_blockhash().await?;
        let tx =
            Transaction::new_signed_with_payer(&[instr], Some(&self.pubkey), &[&kp], blockhash);

        // Send and wait for confirmation
        let _ = self.client.send_and_confirm_transaction(&tx).await?;

        Ok(())
    }

    pub async fn create_token_account(&self, mint_str: &str) -> Result<Pubkey> {
        let mint = Pubkey::from_str(mint_str)?;

        let (token_account, _) = Pubkey::find_program_address(
            &[
                &self.pubkey.to_bytes(),
                &TOKEN_ID.to_bytes(),
                &mint.to_bytes(),
            ],
            &ATOKEN_ID,
        );

        let instr = Instruction {
            program_id: ATOKEN_ID,
            accounts: vec![
                AccountMeta::new(self.pubkey, true),
                AccountMeta::new(token_account, false),
                AccountMeta::new_readonly(self.pubkey, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(SYSTEM_ID, false),
                AccountMeta::new_readonly(TOKEN_ID, false),
            ],
            data: vec![0],
        };

        self.build_sign_and_send(instr).await?;

        Ok(token_account)
    }

    pub async fn close_token_account(&self, mint_str: &str) -> Result<()> {
        let mint = Pubkey::from_str(mint_str)?;
        let token_account = self.get_token_account(&mint).await?;

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

        self.build_sign_and_send(instr).await?;

        Ok(())
    }

    async fn get_token_account(&self, mint: &Pubkey) -> Result<Pubkey> {
        // Get token account
        let accounts = self
            .client
            .get_token_accounts_by_owner(&self.pubkey, Mint(*mint))
            .await?;
        let token = accounts.first().context("Could not find token account")?;

        // Get token account pubkey
        let addy = Pubkey::from_str_const(&token.pubkey);

        Ok(addy)
    }

    async fn make_txn_receipt(&self, operation: CryptoOperation, sig: Signature) -> Result<Txn> {
        let data = self
            .client
            .get_transaction(&sig, UiTransactionEncoding::Json)
            .await?;

        let gas_used = (data.transaction.meta.unwrap().fee as f64) / 1e9;

        let txn = Txn {
            pubkey: self.pubkey.to_string(),
            block: data.slot,
            timestamp: data.block_time.unwrap().try_into()?,
            gas_used,
            operation,
        };

        Ok(txn)
    }

    pub fn get_pubkey(&self) -> Result<String> {
        Ok(self.pubkey.to_string())
    }

    // fn get_signer(&self) -> Result<CryptoSigners> {
    //     let kp = self.key_pair.as_ref().unwrap();
    //     Ok(CryptoSigners::Sol(kp.insecure_clone()))
    // }

    pub fn parse_native_amount(&self, amount: f64) -> Result<u64> {
        let amt = (amount * 1e9) as u64;

        Ok(amt)
    }

    pub async fn parse_token_amount(&self, amount: f64, token: &str) -> Result<u64> {
        let pubkey = Pubkey::from_str_const(token);
        let token_addy = self.get_token_account(&pubkey).await?;
        let acct = self.client.get_token_account(&token_addy).await?.unwrap();

        let deci = acct.token_amount.decimals;

        let amt = (amount * 10.0_f64.powi(deci.into())) as u64;

        Ok(amt)
    }

    pub async fn close(&self, to: &str) -> Result<()> {
        let bal = self.balance().await?;

        // Subtract rent and fee buffer
        // TODO: find out how to not leave rent behind
        let amount = bal - 0.00205;

        let _ = self.transfer(to, amount).await?;

        Ok(())
    }

    // async fn get_history(&self) -> Result<Vec<(String, Txn)>> {
    //     let tmp = Vec::new();

    //     Ok(tmp)
    // }

    pub async fn balance(&self) -> Result<f64> {
        let balance = self.client.get_balance(&self.pubkey).await?;
        let bal = (balance as f64) / 1e9;

        Ok(bal)
    }

    pub async fn transfer(&self, to: &str, amount: f64) -> Result<(String, Txn)> {
        let kp = self.key_pair.as_ref().unwrap();
        let to_pubkey = Pubkey::from_str_const(to);
        let lamp = self.parse_native_amount(amount)?;

        let info = transfer(&self.pubkey, &to_pubkey, lamp);
        let mut trans = Transaction::new_with_payer(&[info], Some(&self.pubkey));

        let blockhash = self.client.get_latest_blockhash().await?;
        trans.sign(&[kp], blockhash);

        let sig = self
            .client
            .send_and_confirm_transaction(&trans)
            .await
            .unwrap();

        let hash = sig.to_string();

        let operation = CryptoOperation::Transfer(CryptoTransfer {
            token: "SOL".to_string(),
            amount,
            from: self.pubkey.to_string(),
            to: to.to_string(),
        });

        let txn = self.make_txn_receipt(operation, sig).await?;

        Ok((hash, txn))
    }

    pub async fn token_balance(&self, mint: &str) -> Result<f64> {
        let mint_pubkey = Pubkey::from_str_const(mint);
        let addy_result = self.get_token_account(&mint_pubkey).await;

        let bal = match addy_result {
            Ok(addy) => {
                let token_data = self.client.get_token_account_balance(&addy).await?;
                token_data.ui_amount.unwrap_or(0.0)
            }
            Err(_) => 0.0,
        };

        Ok(bal)
    }

    pub async fn transfer_token(&self, mint: &str, amount: f64, to: &str) -> Result<(String, Txn)> {
        let kp = self.key_pair.as_ref().unwrap();
        let to_pubkey = Pubkey::from_str_const(to);
        let mint_pubkey = Pubkey::from_str_const(mint);
        let source = self.get_token_account(&mint_pubkey).await?;
        let lamp = self.parse_token_amount(amount, mint).await?;

        let mut data = vec![3];
        data.extend_from_slice(&lamp.to_le_bytes());

        let accounts = self
            .client
            .get_token_accounts_by_owner(&to_pubkey, Mint(mint_pubkey))
            .await?;

        let token = accounts.first().unwrap();
        let destination = Pubkey::from_str_const(&token.pubkey);

        let instruction = Instruction {
            program_id: TOKEN_ID,
            accounts: vec![
                AccountMeta::new(source, false),
                AccountMeta::new(destination, false),
                AccountMeta::new_readonly(self.pubkey, true),
            ],
            data,
        };

        let mut trans = Transaction::new_with_payer(&[instruction], Some(&self.pubkey));

        let blockhash = self.client.get_latest_blockhash().await?;
        trans.sign(&[kp], blockhash);

        let sig = self
            .client
            .send_and_confirm_transaction(&trans)
            .await
            .unwrap();

        let hash = sig.to_string();

        let operation = CryptoOperation::Transfer(CryptoTransfer {
            token: "Native".to_string(),
            amount,
            from: self.pubkey.to_string(),
            to: to.to_string(),
        });

        let txn = self.make_txn_receipt(operation, sig).await?;

        Ok((hash, txn))
    }

    pub async fn transfer_all_tokens(&self, mint: &str, to: &str) -> Result<()> {
        let amount = self.token_balance(mint).await?;

        if amount != 0.0 {
            let _ = self.transfer_token(mint, amount, to).await?;
        }

        let _ = self.close_token_account(&mint).await?;

        Ok(())
    }

    pub async fn sign_and_send(&self, mut txn: VersionedTransaction) -> Result<()> {
        let kp = self.key_pair.as_ref().unwrap();
        let message = txn.message.serialize();
        let signature = kp.sign_message(&message);

        if txn.signatures.is_empty() {
            // If no signatures array exists (unlikely with Jupiter)
            txn.signatures.push(signature);
        } else {
            // Replace the first signature (fee payer)
            txn.signatures[0] = signature;
        };

        let _ = self.client.send_and_confirm_transaction(&txn).await?;

        Ok(())
    }
}
