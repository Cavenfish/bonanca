use anyhow::{Context, Result};
use bonanca_keyvault::{hd_keys::HDkeys, keyvault::KeyVault};
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{CommitmentConfig, RpcTransactionConfig, UiTransactionEncoding},
    rpc_response::{OptionSerializer, UiLoadedAddresses},
};
use solana_client::{
    rpc_request::TokenAccountsFilter::Mint, rpc_response::UiTransactionTokenBalance,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    signer::{Signer, keypair::Keypair},
    transaction::{Transaction, VersionedTransaction},
};
use solana_system_interface::instruction::transfer;
use std::{path::Path, str::FromStr};

use crate::{HdWalletLoad, HdWalletView, HdWallets, WalletLoad, WalletView};

const SYSTEM_ID: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
const ATOKEN_ID: Pubkey = Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

impl HdWallets<Keypair, u32> for HDkeys {
    fn get_child_keypair(&self, child: u32) -> Result<Keypair> {
        let path = format!("m/44'/501'/{child}'/0'");
        let secret = self.derive_ed25519_child_prvkey(path)?;
        let keypair = Keypair::new_from_array(secret);
        Ok(keypair)
    }
}

impl HdWallets<Keypair, &str> for HDkeys {
    fn get_child_keypair(&self, path: &str) -> Result<Keypair> {
        let secret = self.derive_ed25519_child_prvkey(path.to_string())?;
        let keypair = Keypair::new_from_array(secret);
        Ok(keypair)
    }
}

pub struct SolWallet {
    pub key_pair: Option<Keypair>,
    pub client: RpcClient,
    pub pubkey: Pubkey,
}

impl WalletView<&str> for SolWallet {
    fn view(pubkey: &str, rpc: &str) -> Self {
        Self {
            key_pair: None,
            client: RpcClient::new(rpc.to_string()),
            pubkey: Pubkey::from_str(pubkey).expect("Could not parse pubkey"),
        }
    }
}

impl WalletLoad<[u8; 32]> for SolWallet {
    fn load(pkey: [u8; 32], rpc: &str) -> Self {
        let kp = Keypair::new_from_array(pkey);
        let client = RpcClient::new(rpc.to_string());
        let pubkey = kp.pubkey();

        Self {
            key_pair: Some(kp),
            client,
            pubkey,
        }
    }
}

impl HdWalletView<&Path, u32> for SolWallet {
    fn view(keyvault: &Path, rpc: &str, child: u32) -> Self {
        let key_vault = KeyVault::load(keyvault);
        let path = format!("m/44'/501'/{child}'/0'");
        let pubkey = key_vault
            .chain_keys
            .get(&path)
            .expect("Child does not exist");
        Self {
            key_pair: None,
            client: RpcClient::new(rpc.to_string()),
            pubkey: Pubkey::from_str(&pubkey).expect("Could not parse pubkey"),
        }
    }
}

impl HdWalletView<&Path, &str> for SolWallet {
    fn view(keyvault: &Path, rpc: &str, path: &str) -> Self {
        let key_vault = KeyVault::load(keyvault);
        let pubkey = key_vault
            .chain_keys
            .get(path)
            .expect("Child does not exist");
        Self {
            key_pair: None,
            client: RpcClient::new(rpc.to_string()),
            pubkey: Pubkey::from_str(&pubkey).expect("Could not parse pubkey"),
        }
    }
}

impl HdWalletLoad<&Path, u32> for SolWallet {
    fn load(keyvault: &Path, rpc: &str, child: u32) -> Self {
        let mut key_vault = KeyVault::load(keyvault);
        let path = format!("m/44'/501'/{child}'/0'");
        let hd_keys = key_vault.decrypt_vault().unwrap();
        let kp: Keypair = hd_keys.get_child_keypair(child).unwrap();
        let client = RpcClient::new(rpc.to_string());
        let pubkey = kp.pubkey();

        // Add pubkey to keyvault if not already in it
        match key_vault.chain_keys.get(&path) {
            Some(_) => {}
            None => {
                key_vault.add_pubkey(&path, &pubkey.to_string());
                key_vault.write(keyvault);
            }
        }

        Self {
            key_pair: Some(kp),
            client,
            pubkey,
        }
    }
}

impl HdWalletLoad<&Path, &str> for SolWallet {
    fn load(keyvault: &Path, rpc: &str, path: &str) -> Self {
        let mut key_vault = KeyVault::load(keyvault);
        let hd_keys = key_vault.decrypt_vault().unwrap();
        let kp: Keypair = hd_keys.get_child_keypair(path).unwrap();
        let client = RpcClient::new(rpc.to_string());
        let pubkey = kp.pubkey();

        // Add pubkey to keyvault if not already in it
        match key_vault.chain_keys.get(path) {
            Some(_) => {}
            None => {
                key_vault.add_pubkey(&path, &pubkey.to_string());
                key_vault.write(keyvault);
            }
        }

        Self {
            key_pair: Some(kp),
            client,
            pubkey,
        }
    }
}

impl SolWallet {
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

    pub async fn get_timestamp(&self) -> Result<i64> {
        let slot = self.client.get_slot().await?;
        let time = self.client.get_block_time(slot).await?;
        Ok(time)
    }

    pub async fn get_ata(&self, mint: &str) -> Result<Pubkey> {
        let token = Pubkey::from_str(mint)?;
        let owner = self.client.get_account(&token).await?.owner;

        let (token_account, _) = Pubkey::find_program_address(
            &[
                &self.pubkey.to_bytes(),
                &owner.to_bytes(),
                &token.to_bytes(),
            ],
            &ATOKEN_ID,
        );

        Ok(token_account)
    }

    pub async fn create_token_account(&self, mint: &str) -> Result<Pubkey> {
        let token = Pubkey::from_str(mint)?;
        let owner = self.client.get_account(&token).await?.owner;

        let (token_account, _) = Pubkey::find_program_address(
            &[
                &self.pubkey.to_bytes(),
                &owner.to_bytes(),
                &token.to_bytes(),
            ],
            &ATOKEN_ID,
        );

        let instr = Instruction {
            program_id: ATOKEN_ID,
            accounts: vec![
                AccountMeta::new(self.pubkey, true),
                AccountMeta::new(token_account, false),
                AccountMeta::new_readonly(self.pubkey, false),
                AccountMeta::new_readonly(token, false),
                AccountMeta::new_readonly(SYSTEM_ID, false),
                AccountMeta::new_readonly(owner, false),
            ],
            data: vec![0],
        };

        self.build_sign_and_send(instr).await?;

        Ok(token_account)
    }

    pub async fn close_token_account(&self, mint_str: &str) -> Result<()> {
        let mint = Pubkey::from_str(mint_str)?;
        let owner = self.client.get_account(&mint).await?.owner;
        let token_account = self.get_token_account(&mint).await?;

        // Build close instructions
        let instr = Instruction {
            program_id: owner,
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

    pub fn get_pubkey(&self) -> Result<String> {
        Ok(self.pubkey.to_string())
    }

    pub fn format_native(&self, amount: f64) -> Result<u64> {
        let amt = (amount * 1e9) as u64;

        Ok(amt)
    }

    pub fn parse_native(&self, amount: u64) -> Result<f64> {
        Ok((amount as f64) / 1.0e9)
    }

    pub async fn format_token(&self, amount: f64, token: &str) -> Result<u64> {
        let pubkey = Pubkey::from_str_const(token);
        let token_addy = self.get_token_account(&pubkey).await?;
        let acct = self.client.get_token_account(&token_addy).await?.unwrap();

        let deci = acct.token_amount.decimals;

        let amt = (amount * 10.0_f64.powi(deci.into())) as u64;

        Ok(amt)
    }

    pub async fn parse_token(&self, amount: u64, token: &str) -> Result<f64> {
        let pubkey = Pubkey::from_str_const(token);
        let token_addy = self.get_token_account(&pubkey).await?;
        let acct = self.client.get_token_account(&token_addy).await?.unwrap();

        let deci = acct.token_amount.decimals;

        Ok((amount as f64) / 10.0_f64.powi(deci.into()))
    }

    pub async fn close(&self, to: &str) -> Result<()> {
        let bal = self.balance().await?;

        // Subtract fee
        let amount = bal - 0.000005;

        let _ = self.transfer(to, amount).await?;

        Ok(())
    }

    pub async fn balance(&self) -> Result<f64> {
        let balance = self.client.get_balance(&self.pubkey).await?;
        let bal = (balance as f64) / 1e9;

        Ok(bal)
    }

    pub async fn transfer(&self, to: &str, amount: f64) -> Result<SolTxnReceipt> {
        let kp = self.key_pair.as_ref().unwrap();
        let to_pubkey = Pubkey::from_str_const(to);
        let lamp = self.format_native(amount)?;

        let info = transfer(&self.pubkey, &to_pubkey, lamp);
        let mut trans = Transaction::new_with_payer(&[info], Some(&self.pubkey));

        let blockhash = self.client.get_latest_blockhash().await?;
        trans.sign(&[kp], blockhash);

        let sig = self
            .client
            .send_and_confirm_transaction(&trans)
            .await
            .unwrap();

        Ok(SolTxnReceipt::new(sig, &self.client).await)
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

    pub async fn burn_token(&self, mint: &str, amount: f64) -> Result<SolTxnReceipt> {
        let kp = self.key_pair.as_ref().unwrap();
        let mint_pubkey = Pubkey::from_str_const(mint);
        let owner = self.client.get_account(&mint_pubkey).await?.owner;
        let source = self.get_token_account(&mint_pubkey).await?;
        let lamp = self.format_token(amount, mint).await?;

        let mut data = vec![8];
        data.extend_from_slice(&lamp.to_le_bytes());

        let instruction = Instruction {
            program_id: owner,
            accounts: vec![
                AccountMeta::new(source, false),
                AccountMeta::new(mint_pubkey, false),
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

        Ok(SolTxnReceipt::new(sig, &self.client).await)
    }

    pub async fn transfer_token(&self, mint: &str, amount: f64, to: &str) -> Result<SolTxnReceipt> {
        let kp = self.key_pair.as_ref().unwrap();
        let to_pubkey = Pubkey::from_str_const(to);
        let mint_pubkey = Pubkey::from_str_const(mint);
        let owner = self.client.get_account(&mint_pubkey).await?.owner;
        let source = self.get_token_account(&mint_pubkey).await?;
        let lamp = self.format_token(amount, mint).await?;

        let mut data = vec![3];
        data.extend_from_slice(&lamp.to_le_bytes());

        let accounts = self
            .client
            .get_token_accounts_by_owner(&to_pubkey, Mint(mint_pubkey))
            .await?;

        let token = accounts.first().unwrap();
        let destination = Pubkey::from_str_const(&token.pubkey);

        let instruction = Instruction {
            program_id: owner,
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

        Ok(SolTxnReceipt::new(sig, &self.client).await)
    }

    pub async fn transfer_all_tokens(&self, mint: &str, to: &str) -> Result<()> {
        let amount = self.token_balance(mint).await?;

        if amount != 0.0 {
            let _ = self.transfer_token(mint, amount, to).await?;
        }

        let _ = self.close_token_account(&mint).await?;

        Ok(())
    }

    pub async fn sign_and_send(&self, mut txn: VersionedTransaction) -> Result<SolTxnReceipt> {
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

        let hash = self.client.get_latest_blockhash().await?;
        txn.message.set_recent_blockhash(hash);

        let sig = self.client.send_and_confirm_transaction(&txn).await?;

        Ok(SolTxnReceipt::new(sig, &self.client).await)
    }
}

pub struct SolTxnReceipt {
    pub hash: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub gas_used: f64,
    pub pre_balances: Vec<f64>,
    pub post_balances: Vec<f64>,
    pub pre_token_balances: Option<Vec<UiTransactionTokenBalance>>,
    pub post_token_balances: Option<Vec<UiTransactionTokenBalance>>,
    pub loaded_addresses: OptionSerializer<UiLoadedAddresses>,
}

impl SolTxnReceipt {
    pub async fn new(sig: Signature, client: &RpcClient) -> Self {
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        };

        let data = client
            .get_transaction_with_config(&sig, config)
            .await
            .expect("Transaction not found");

        let meta = data.transaction.meta.unwrap();
        let gas_used = (meta.fee as f64) / 1e9;
        let pre_balances = meta
            .pre_balances
            .iter()
            .map(|b| (*b as f64) / 1e9)
            .collect();
        let post_balances = meta
            .post_balances
            .iter()
            .map(|b| (*b as f64) / 1e9)
            .collect();

        Self {
            hash: sig.to_string(),
            slot: data.slot,
            block_time: data.block_time,
            gas_used,
            pre_balances,
            post_balances,
            pre_token_balances: meta.pre_token_balances.into(),
            post_token_balances: meta.post_token_balances.into(),
            loaded_addresses: meta.loaded_addresses,
        }
    }
}
