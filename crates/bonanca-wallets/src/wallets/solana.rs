//! Solana wallet implementation.
//!
//! [`SolWallet`] wraps a [`RpcClient`] plus an optional [`Keypair`]. View-only wallets hold
//! just a [`Pubkey`] and can read balances and derive token accounts; loaded wallets also
//! carry a keypair so they can sign and send transactions.

use std::{path::Path, str::FromStr};

use bonanca_keyvault::{KeyVaultErrors, hd_keys::HDkeys, keyvault::KeyVault};
use solana_client::{
    client_error::ClientError as SolanaClientError,
    nonblocking::rpc_client::RpcClient,
    rpc_config::{CommitmentConfig, RpcTransactionConfig, UiTransactionEncoding},
    rpc_request::TokenAccountsFilter::Mint,
    rpc_response::{OptionSerializer, UiLoadedAddresses, UiTransactionTokenBalance},
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::{ParsePubkeyError, Pubkey},
    signature::Signature,
    signer::{Signer, SignerError as SolanaSignerError, keypair::Keypair},
    transaction::{Transaction, VersionedTransaction},
};
use solana_system_interface::instruction::transfer;
use thiserror::Error;

use crate::{
    GeneralWalletErrors, HdWalletLoad, HdWalletView, HdWallets, WalletErrors, WalletLoad,
    WalletView,
};

impl From<ParsePubkeyError> for WalletErrors {
    fn from(err: ParsePubkeyError) -> Self {
        Self::Sol(SolWalletErrors::InvalidPubkey(err))
    }
}

/// Errors produced while interacting with a [`SolWallet`].
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum SolWalletErrors {
    /// Errors common to all wallets (keys, vault, signer status, URL parsing).
    #[error(transparent)]
    General(#[from] GeneralWalletErrors),

    /// A Solana public key string is invalid or malformed.
    #[error("invalid Solana public key: {0}")]
    InvalidPubkey(#[from] ParsePubkeyError),

    /// An RPC or network error occurred when interacting with the Solana cluster.
    #[error("Solana client error: {0}")]
    Client(#[from] SolanaClientError),

    /// An error occurred while signing a Solana transaction or message.
    #[error("Solana signing error: {0}")]
    Signer(#[from] SolanaSignerError),

    /// The requested SPL token account was not found on chain for the given mint or owner.
    #[error("token account not found for mint: {0}")]
    TokenAccountNotFound(String),

    /// The requested on-chain account does not exist or contains no data.
    #[error("account not found: {0}")]
    AccountNotFound(String),

    /// The transaction receipt could not be found for the given signature or hash.
    #[error("transaction not found: {0}")]
    TransactionNotFound(String),

    /// The confirmed transaction did not include execution metadata or receipt details.
    #[error("transaction execution metadata is missing")]
    MissingTransactionMeta,
}

impl From<KeyVaultErrors> for SolWalletErrors {
    fn from(err: KeyVaultErrors) -> Self {
        Self::General(GeneralWalletErrors::KeyVault(err))
    }
}

const SYSTEM_ID: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
const ATOKEN_ID: Pubkey = Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

impl HdWallets<Keypair, u32> for HDkeys {
    fn get_child_keypair(&self, child: u32) -> Result<Keypair, WalletErrors> {
        let path = format!("m/44'/501'/{child}'/0'");
        let secret = self.derive_ed25519_child_prvkey(path)?;
        let keypair = Keypair::new_from_array(*secret);
        Ok(keypair)
    }
}

impl HdWallets<Keypair, &str> for HDkeys {
    fn get_child_keypair(&self, path: &str) -> Result<Keypair, WalletErrors> {
        let secret = self.derive_ed25519_child_prvkey(path.to_string())?;
        let keypair = Keypair::new_from_array(*secret);
        Ok(keypair)
    }
}

/// A Solana wallet supporting view-only and transacting modes.
pub struct SolWallet {
    /// Optional keypair used to sign transactions. Present for loaded wallets, `None` for
    /// view-only wallets.
    pub key_pair: Option<Keypair>,
    /// RPC client used to query the cluster and send transactions.
    pub client: RpcClient,
    /// Public key (address) of this wallet.
    pub pubkey: Pubkey,
}

impl WalletView<&str> for SolWallet {
    fn view(pubkey: &str, rpc: &str) -> Result<Self, WalletErrors> {
        Ok(Self {
            key_pair: None,
            client: RpcClient::new(rpc.to_string()),
            pubkey: Pubkey::from_str(pubkey)?,
        })
    }
}

impl WalletLoad<[u8; 32]> for SolWallet {
    fn load(pkey: [u8; 32], rpc: &str) -> Result<Self, WalletErrors> {
        let kp = Keypair::new_from_array(pkey);
        let client = RpcClient::new(rpc.to_string());
        let pubkey = kp.pubkey();

        Ok(Self { key_pair: Some(kp), client, pubkey })
    }
}

impl<T: AsRef<Path>> HdWalletView<T, u32> for SolWallet {
    fn view(keyvault: T, rpc: &str, child: u32) -> Result<Self, WalletErrors> {
        let key_vault = KeyVault::load(keyvault.as_ref())?;
        let path = format!("m/44'/501'/{child}'/0'");
        let pubkey = key_vault.get_pubkey(&path)?;
        Ok(Self {
            key_pair: None,
            client: RpcClient::new(rpc.to_string()),
            pubkey: Pubkey::from_str(pubkey)?,
        })
    }
}

impl<T: AsRef<Path>> HdWalletView<T, &str> for SolWallet {
    fn view(keyvault: T, rpc: &str, path: &str) -> Result<Self, WalletErrors> {
        let key_vault = KeyVault::load(keyvault.as_ref())?;
        let pubkey = key_vault.get_pubkey(path)?;
        Ok(Self {
            key_pair: None,
            client: RpcClient::new(rpc.to_string()),
            pubkey: Pubkey::from_str(pubkey)?,
        })
    }
}

impl<T: AsRef<Path>> HdWalletLoad<T, u32> for SolWallet {
    fn load(keyvault: T, rpc: &str, child: u32, password: &str) -> Result<Self, WalletErrors> {
        let mut key_vault = KeyVault::load(keyvault.as_ref())?;
        let path = format!("m/44'/501'/{child}'/0'");
        let hd_keys = key_vault.decrypt_vault(password)?;
        let kp: Keypair = hd_keys.get_child_keypair(child)?;
        let client = RpcClient::new(rpc.to_string());
        let pubkey = kp.pubkey();

        // Add pubkey to keyvault if not already in it
        match key_vault.chain_keys.get(&path) {
            Some(_) => {}
            None => {
                key_vault.add_pubkey(&path, &pubkey.to_string());
                key_vault.write(keyvault.as_ref())?;
            }
        }

        Ok(Self { key_pair: Some(kp), client, pubkey })
    }
}

impl<T: AsRef<Path>> HdWalletLoad<T, &str> for SolWallet {
    fn load(keyvault: T, rpc: &str, path: &str, password: &str) -> Result<Self, WalletErrors> {
        let mut key_vault = KeyVault::load(keyvault.as_ref())?;
        let hd_keys = key_vault.decrypt_vault(password)?;
        let kp: Keypair = hd_keys.get_child_keypair(path)?;
        let client = RpcClient::new(rpc.to_string());
        let pubkey = kp.pubkey();

        // Add pubkey to keyvault if not already in it
        match key_vault.chain_keys.get(path) {
            Some(_) => {}
            None => {
                key_vault.add_pubkey(path, &pubkey.to_string());
                key_vault.write(keyvault.as_ref())?;
            }
        }

        Ok(Self { key_pair: Some(kp), client, pubkey })
    }
}

impl SolWallet {
    fn get_keypair(&self) -> Result<&Keypair, SolWalletErrors> {
        match self.key_pair.as_ref() {
            Some(kp) => return Ok(kp),
            None => Err(SolWalletErrors::General(GeneralWalletErrors::MissingSigner)),
        }
    }

    async fn build_sign_and_send(&self, instr: Instruction) -> Result<(), SolWalletErrors> {
        let kp = self.get_keypair()?;

        // Get blockhash and sign transaction
        let blockhash = self.client.get_latest_blockhash().await?;
        let tx =
            Transaction::new_signed_with_payer(&[instr], Some(&self.pubkey), &[&kp], blockhash);

        // Send and wait for confirmation
        let _ = self.client.send_and_confirm_transaction(&tx).await?;

        Ok(())
    }

    /// Returns the Unix block time of the current slot.
    pub async fn get_timestamp(&self) -> Result<i64, SolWalletErrors> {
        let slot = self.client.get_slot().await?;
        let time = self.client.get_block_time(slot).await?;
        Ok(time)
    }

    /// Derives the associated token account (ATA) address for the SPL token `mint` and this
    /// wallet, without creating it. The mint's owning token program is used in the derivation.
    pub async fn get_ata(&self, mint: &str) -> Result<Pubkey, SolWalletErrors> {
        let token = Pubkey::from_str(mint)?;
        let owner = self.client.get_account(&token).await?.owner;

        let (token_account, _) = Pubkey::find_program_address(
            &[&self.pubkey.to_bytes(), &owner.to_bytes(), &token.to_bytes()],
            &ATOKEN_ID,
        );

        Ok(token_account)
    }

    /// Creates this wallet's associated token account for `mint` and returns its address.
    ///
    /// Requires a loaded wallet; a view-only wallet cannot sign the creation transaction and
    /// returns an error instead.
    pub async fn create_token_account(&self, mint: &str) -> Result<Pubkey, SolWalletErrors> {
        let token = Pubkey::from_str(mint)?;
        let owner = self.client.get_account(&token).await?.owner;

        let (token_account, _) = Pubkey::find_program_address(
            &[&self.pubkey.to_bytes(), &owner.to_bytes(), &token.to_bytes()],
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

    /// Closes this wallet's associated token account for `mint`, refunding its rent to the
    /// wallet. The account must have a zero token balance.
    ///
    /// Requires a loaded wallet; a view-only wallet cannot sign the closing transaction and
    /// returns an error instead.
    pub async fn close_token_account(&self, mint_str: &str) -> Result<(), SolWalletErrors> {
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

    async fn get_token_account(&self, mint: &Pubkey) -> Result<Pubkey, SolWalletErrors> {
        // Get token account
        let accounts = self.client.get_token_accounts_by_owner(&self.pubkey, Mint(*mint)).await?;

        match accounts.first() {
            Some(token) => {
                let addy = Pubkey::from_str_const(&token.pubkey);
                return Ok(addy);
            }
            None => Err(SolWalletErrors::AccountNotFound(mint.to_string())),
        }
    }

    /// Returns the wallet's public key as a base58 string.
    pub fn get_pubkey(&self) -> Result<String, SolWalletErrors> {
        Ok(self.pubkey.to_string())
    }

    /// Converts a SOL amount in `f64` to lamports (`u64`), truncating fractional lamports.
    pub fn format_native(&self, amount: f64) -> Result<u64, SolWalletErrors> {
        let amt = (amount * 1e9) as u64;

        Ok(amt)
    }

    /// Converts lamports (`u64`) to a SOL amount in `f64`.
    pub fn parse_native(&self, amount: u64) -> Result<f64, SolWalletErrors> {
        Ok((amount as f64) / 1.0e9)
    }

    /// Converts a human-readable amount of the SPL token `mint` to raw token units (`u64`),
    /// using the decimals of this wallet's token account for that mint.
    pub async fn format_token(&self, amount: f64, token: &str) -> Result<u64, SolWalletErrors> {
        let pubkey = Pubkey::from_str_const(token);
        let token_addy = self.get_token_account(&pubkey).await?;
        let acct = self
            .client
            .get_token_account(&token_addy)
            .await?
            .ok_or(SolWalletErrors::AccountNotFound(token.to_string()))?;

        let deci = acct.token_amount.decimals;

        let amt = (amount * 10.0_f64.powi(deci.into())) as u64;

        Ok(amt)
    }

    /// Converts raw units of the SPL token `mint` to a human-readable amount in `f64`, using the
    /// decimals of this wallet's token account for that mint.
    pub async fn parse_token(&self, amount: u64, token: &str) -> Result<f64, SolWalletErrors> {
        let pubkey = Pubkey::from_str_const(token);
        let token_addy = self.get_token_account(&pubkey).await?;
        let acct = self
            .client
            .get_token_account(&token_addy)
            .await?
            .ok_or(SolWalletErrors::AccountNotFound(token.to_string()))?;

        let deci = acct.token_amount.decimals;

        Ok((amount as f64) / 10.0_f64.powi(deci.into()))
    }

    /// Transfers nearly the entire SOL balance to `to`, keeping `0.000005` SOL back for the
    /// transaction fee.
    pub async fn close(&self, to: &str) -> Result<(), SolWalletErrors> {
        let bal = self.balance().await?;

        // Subtract fee
        let amount = bal - 0.000005;

        let _ = self.transfer(to, amount).await?;

        Ok(())
    }

    /// Fetches the wallet's SOL balance as an `f64`.
    pub async fn balance(&self) -> Result<f64, SolWalletErrors> {
        let balance = self.client.get_balance(&self.pubkey).await?;
        let bal = (balance as f64) / 1e9;

        Ok(bal)
    }

    /// Transfers `amount` SOL to `to` and returns the confirmed transaction receipt.
    pub async fn transfer(&self, to: &str, amount: f64) -> Result<SolTxnReceipt, SolWalletErrors> {
        let kp = self.get_keypair()?;
        let to_pubkey = Pubkey::from_str_const(to);
        let lamp = self.format_native(amount)?;

        let info = transfer(&self.pubkey, &to_pubkey, lamp);
        let mut trans = Transaction::new_with_payer(&[info], Some(&self.pubkey));

        let blockhash = self.client.get_latest_blockhash().await?;
        trans.sign(&[kp], blockhash);

        let sig = self.client.send_and_confirm_transaction(&trans).await?;

        Ok(SolTxnReceipt::new(sig, &self.client).await)
    }

    /// Fetches the wallet's balance of the SPL token `mint` as an `f64`, returning `0.0` when
    /// the wallet has no token account for that mint.
    pub async fn token_balance(&self, mint: &str) -> Result<f64, SolWalletErrors> {
        let mint_pubkey = Pubkey::from_str_const(mint);
        let addy = self.get_token_account(&mint_pubkey).await?;

        let token_data = self.client.get_token_account_balance(&addy).await?;

        let bal = token_data.ui_amount.unwrap_or(0.0);

        Ok(bal)
    }

    /// Burns `amount` of the SPL token `mint` from the wallet's token account and returns the
    /// confirmed transaction receipt.
    pub async fn burn_token(
        &self,
        mint: &str,
        amount: f64,
    ) -> Result<SolTxnReceipt, SolWalletErrors> {
        let kp = self.get_keypair()?;
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

        let sig = self.client.send_and_confirm_transaction(&trans).await?;

        Ok(SolTxnReceipt::new(sig, &self.client).await)
    }

    /// Transfers `amount` of the SPL token `mint` to `to` and returns the confirmed transaction
    /// receipt. `to` must already have a token account for `mint`.
    pub async fn transfer_token(
        &self,
        mint: &str,
        amount: f64,
        to: &str,
    ) -> Result<SolTxnReceipt, SolWalletErrors> {
        let kp = self.get_keypair()?;
        let to_pubkey = Pubkey::from_str_const(to);
        let mint_pubkey = Pubkey::from_str_const(mint);
        let owner = self.client.get_account(&mint_pubkey).await?.owner;
        let source = self.get_token_account(&mint_pubkey).await?;
        let lamp = self.format_token(amount, mint).await?;

        let mut data = vec![3];
        data.extend_from_slice(&lamp.to_le_bytes());

        let accounts =
            self.client.get_token_accounts_by_owner(&to_pubkey, Mint(mint_pubkey)).await?;

        let token = accounts.first().ok_or(SolWalletErrors::AccountNotFound(mint.to_string()))?;
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

        let sig = self.client.send_and_confirm_transaction(&trans).await?;

        Ok(SolTxnReceipt::new(sig, &self.client).await)
    }

    /// Transfers the wallet's entire balance of the SPL token `mint` to `to`, then closes the
    /// wallet's token account for that mint.
    pub async fn transfer_all_tokens(&self, mint: &str, to: &str) -> Result<(), SolWalletErrors> {
        let amount = self.token_balance(mint).await?;

        if amount != 0.0 {
            let _ = self.transfer_token(mint, amount, to).await?;
        }

        let _ = self.close_token_account(&mint).await?;

        Ok(())
    }

    /// Signs an externally built [`VersionedTransaction`] with this wallet's keypair, replaces
    /// the fee-payer signature (the first signature slot) with that signature, refreshes the
    /// recent blockhash, sends it, and returns the confirmed transaction receipt.
    pub async fn sign_and_send(
        &self,
        mut txn: VersionedTransaction,
    ) -> Result<SolTxnReceipt, SolWalletErrors> {
        let kp = self.get_keypair()?;
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

/// Details of a confirmed Solana transaction, returned by the transacting [`SolWallet`] methods.
pub struct SolTxnReceipt {
    /// Base58 transaction signature.
    pub hash: String,
    /// Slot in which the transaction was confirmed.
    pub slot: u64,
    /// Unix block time of the confirmation slot, if the cluster reported one.
    pub block_time: Option<i64>,
    /// Transaction fee in SOL.
    pub gas_used: f64,
    /// Account balances in SOL before the transaction, in account order.
    pub pre_balances: Vec<f64>,
    /// Account balances in SOL after the transaction, in account order.
    pub post_balances: Vec<f64>,
    /// SPL token balances before the transaction, if reported by the cluster.
    pub pre_token_balances: Option<Vec<UiTransactionTokenBalance>>,
    /// SPL token balances after the transaction, if reported by the cluster.
    pub post_token_balances: Option<Vec<UiTransactionTokenBalance>>,
    /// Address lookup table addresses loaded by the transaction, if any.
    pub loaded_addresses: OptionSerializer<UiLoadedAddresses>,
}

impl SolTxnReceipt {
    /// Fetches the confirmed transaction identified by `sig` and builds a receipt from it.
    ///
    /// # Panics
    ///
    /// Panics if the cluster does not return the transaction or its execution metadata.
    pub async fn new(sig: Signature, client: &RpcClient) -> Self {
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        };

        let data =
            client.get_transaction_with_config(&sig, config).await.expect("Transaction not found");

        let meta = data.transaction.meta.unwrap();
        let gas_used = (meta.fee as f64) / 1e9;
        let pre_balances = meta.pre_balances.iter().map(|b| (*b as f64) / 1e9).collect();
        let post_balances = meta.post_balances.iter().map(|b| (*b as f64) / 1e9).collect();

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
