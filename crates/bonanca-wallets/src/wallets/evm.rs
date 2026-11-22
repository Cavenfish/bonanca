use std::{num::ParseFloatError, path::Path, str::FromStr};

use alloy::{
    contract::Error as ContractError,
    hex::FromHexError,
    network::TransactionBuilder,
    providers::{DynProvider, PendingTransactionError, Provider, ProviderBuilder},
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::{
        Error as SignerError, Signer,
        k256::ecdsa::SigningKey,
        local::{LocalSigner, PrivateKeySigner},
    },
    sol,
    transports::{TransportError, http::reqwest::Url},
};
use alloy_primitives::{
    Address, AddressError, FixedBytes, Signature, U256, Uint,
    utils::{UnitsError, format_ether, format_units, parse_ether, parse_units},
};
use bonanca_keyvault::{KeyVaultErrors, hd_keys::HDkeys, keyvault::KeyVault};
use thiserror::Error;

use crate::{
    GeneralWalletErrors, HdWalletLoad, HdWalletView, HdWallets, WalletErrors, WalletLoad,
    WalletView,
};

/// Errors produced while interacting with an [`EvmWallet`].
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum EvmWalletErrors {
    /// Errors common to all wallets (keys, vault, signer status, URL parsing).
    #[error(transparent)]
    General(#[from] GeneralWalletErrors),

    /// An EVM address string is invalid or malformed.
    #[error("invalid EVM address: {0}")]
    InvalidAddress(#[from] AddressError),

    /// Parsing or formatting token or native currency units failed.
    #[error("unit conversion error: {0}")]
    Units(#[from] UnitsError),

    /// An error occurred during transport or RPC communication with the EVM node.
    #[error("EVM transport error: {0}")]
    Transport(#[from] TransportError),

    /// An error occurred while interacting with an EVM smart contract.
    #[error("contract interaction error: {0}")]
    Contract(#[from] ContractError),

    /// An error occurred while waiting for an EVM transaction receipt or confirmation.
    #[error("pending transaction error: {0}")]
    PendingTransaction(#[from] PendingTransactionError),

    /// An error occurred while signing an EVM hash or transaction.
    #[error("EVM signing error: {0}")]
    Signer(#[from] SignerError),
}

impl From<KeyVaultErrors> for EvmWalletErrors {
    fn from(err: KeyVaultErrors) -> Self {
        Self::General(GeneralWalletErrors::KeyVault(err))
    }
}

impl From<FromHexError> for EvmWalletErrors {
    fn from(err: FromHexError) -> Self {
        Self::InvalidAddress(AddressError::Hex(err))
    }
}

impl From<ParseFloatError> for EvmWalletErrors {
    fn from(err: ParseFloatError) -> Self {
        Self::General(GeneralWalletErrors::ParseFloat(err))
    }
}

impl HdWallets<LocalSigner<SigningKey>, u32> for HDkeys {
    fn get_child_keypair(&self, child: u32) -> Result<LocalSigner<SigningKey>, WalletErrors> {
        let path = format!("m/44'/60'/{child}'/0/0");
        let secret = self.derive_secp256k1_child_prvkey(path)?;
        let key_bytes = FixedBytes::new(*secret);
        let signer = PrivateKeySigner::from_bytes(&key_bytes)
            .map_err(|_| GeneralWalletErrors::InvalidPrivateKey)?;
        Ok(signer)
    }
}

impl HdWallets<LocalSigner<SigningKey>, &str> for HDkeys {
    fn get_child_keypair(&self, path: &str) -> Result<LocalSigner<SigningKey>, WalletErrors> {
        let secret = self.derive_secp256k1_child_prvkey(path.to_string())?;
        let key_bytes = FixedBytes::new(*secret);
        let signer = PrivateKeySigner::from_bytes(&key_bytes)
            .map_err(|_| GeneralWalletErrors::InvalidPrivateKey)?;
        Ok(signer)
    }
}

// ABI for smart contracts
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "src/wallets/ABI/ERC20.json"
}

/// Ethereum Virtual Machine (EVM) wallet instance supporting view-only and transacting modes.
pub struct EvmWallet {
    /// Optional local private key signer. Present for loaded wallets, `None` for view-only
    /// wallets.
    pub signer: Option<LocalSigner<SigningKey>>,
    /// RPC provider for querying state and broadcasting transactions.
    pub client: DynProvider,
    /// Public Ethereum address of this wallet.
    pub pubkey: Address,
}

impl WalletView<&str> for EvmWallet {
    fn view(pubkey: &str, rpc: &str) -> Result<Self, WalletErrors> {
        let rpc_url =
            Url::parse(rpc).map_err(|_| GeneralWalletErrors::InvalidRpcUrl(rpc.into()))?;
        let client: DynProvider = ProviderBuilder::new().connect_http(rpc_url).erased();
        let pubkey = Address::from_str(pubkey).map_err(|err| EvmWalletErrors::from(err))?;

        Ok(Self { signer: None, client, pubkey })
    }
}

impl WalletLoad<[u8; 32]> for EvmWallet {
    fn load(pkey: [u8; 32], rpc: &str) -> Result<Self, WalletErrors> {
        let key_bytes = FixedBytes::new(pkey);
        let signer = PrivateKeySigner::from_bytes(&key_bytes)
            .map_err(|_| GeneralWalletErrors::InvalidPrivateKey)?;
        let rpc_url =
            Url::parse(rpc).map_err(|_| GeneralWalletErrors::InvalidRpcUrl(rpc.into()))?;
        let pubkey = signer.address();
        let client: DynProvider =
            ProviderBuilder::new().wallet(signer.clone()).connect_http(rpc_url).erased();

        Ok(Self { signer: Some(signer), client, pubkey })
    }
}

impl<T: AsRef<Path>> HdWalletView<T, u32> for EvmWallet {
    fn view(keyvault: T, rpc: &str, child: u32) -> Result<Self, WalletErrors> {
        let key_vault = KeyVault::load(keyvault.as_ref())?;
        let path = format!("m/44'/60'/{child}'/0/0");
        let pubkey = key_vault.get_pubkey(&path)?;
        let rpc_url =
            Url::parse(rpc).map_err(|_| GeneralWalletErrors::InvalidRpcUrl(rpc.into()))?;
        let pubkey = Address::from_str(pubkey)
            .map_err(|err| EvmWalletErrors::InvalidAddress(AddressError::Hex(err)))?;
        let client: DynProvider = ProviderBuilder::new().connect_http(rpc_url).erased();

        Ok(Self { signer: None, client, pubkey })
    }
}

impl<T: AsRef<Path>> HdWalletView<T, &str> for EvmWallet {
    fn view(keyvault: T, rpc: &str, path: &str) -> Result<Self, WalletErrors> {
        let key_vault = KeyVault::load(keyvault.as_ref())?;
        let pubkey = key_vault.get_pubkey(&path)?;
        let rpc_url =
            Url::parse(rpc).map_err(|_| GeneralWalletErrors::InvalidRpcUrl(rpc.into()))?;
        let pubkey = Address::from_str(pubkey)
            .map_err(|err| EvmWalletErrors::InvalidAddress(AddressError::Hex(err)))?;
        let client: DynProvider = ProviderBuilder::new().connect_http(rpc_url).erased();

        Ok(Self { signer: None, client, pubkey })
    }
}

impl<T: AsRef<Path>> HdWalletLoad<T, u32> for EvmWallet {
    fn load(keyvault: T, rpc: &str, child: u32, password: &str) -> Result<Self, WalletErrors> {
        let mut key_vault = KeyVault::load(keyvault.as_ref())?;
        let path = format!("m/44'/60'/{child}'/0/0");
        let hd_keys = key_vault.decrypt_vault(password)?;
        let signer: LocalSigner<SigningKey> = hd_keys.get_child_keypair(child)?;
        let rpc_url =
            Url::parse(rpc).map_err(|_| GeneralWalletErrors::InvalidRpcUrl(rpc.into()))?;
        let pubkey = signer.address();
        let client: DynProvider =
            ProviderBuilder::new().wallet(signer.clone()).connect_http(rpc_url).erased();

        // Add pubkey to keyvault if not already in it
        match key_vault.chain_keys.get(&path) {
            Some(_) => {}
            None => {
                key_vault.add_pubkey(&path, &pubkey.to_string());
                key_vault.write(keyvault.as_ref())?;
            }
        }

        Ok(Self { signer: Some(signer), client, pubkey })
    }
}

impl<T: AsRef<Path>> HdWalletLoad<T, &str> for EvmWallet {
    fn load(keyvault: T, rpc: &str, path: &str, password: &str) -> Result<Self, WalletErrors> {
        let mut key_vault = KeyVault::load(keyvault.as_ref())?;
        let hd_keys = key_vault.decrypt_vault(password)?;
        let signer: LocalSigner<SigningKey> = hd_keys.get_child_keypair(path)?;
        let rpc_url =
            Url::parse(rpc).map_err(|_| GeneralWalletErrors::InvalidRpcUrl(rpc.into()))?;
        let pubkey = signer.address();
        let client: DynProvider =
            ProviderBuilder::new().wallet(signer.clone()).connect_http(rpc_url).erased();

        // Add pubkey to keyvault if not already in it
        match key_vault.chain_keys.get(path) {
            Some(_) => {}
            None => {
                key_vault.add_pubkey(path, &pubkey.to_string());
                key_vault.write(keyvault.as_ref())?;
            }
        }

        Ok(Self { signer: Some(signer), client, pubkey })
    }
}

impl EvmWallet {
    /// Signs a 32-byte hash using the wallet's private key.
    pub async fn sign_hash(&self, hash: &FixedBytes<32>) -> Result<Signature, EvmWalletErrors> {
        let signer = self.signer.as_ref().ok_or(GeneralWalletErrors::MissingSigner)?;
        let sig = signer.sign_hash(hash).await?;

        Ok(sig)
    }

    /// Approves an ERC-20 token allowance for a spender address.
    pub async fn approve_token_spending(
        &self,
        token: &str,
        spender: &str,
        amount: f64,
    ) -> Result<(), EvmWalletErrors> {
        let _signer = self.signer.as_ref().ok_or(GeneralWalletErrors::MissingSigner)?;
        let token_addy = Address::from_str(token)?;
        let spender_addy = Address::from_str(spender)?;

        let erc20 = ERC20::new(token_addy, &self.client);

        let decimals = erc20.decimals().call().await?;
        let value: Uint<256, 4> = parse_units(&amount.to_string(), decimals)?.into();

        let _ = erc20.approve(spender_addy, value).send().await?.watch().await?;

        Ok(())
    }

    /// Retrieves the current ERC-20 token allowance approved for a spender.
    pub async fn get_token_allowance(
        &self,
        token: &str,
        spender: &str,
    ) -> Result<f64, EvmWalletErrors> {
        let token_addy = Address::from_str(token)?;
        let spender_addy = Address::from_str(spender)?;

        let erc20 = ERC20::new(token_addy, &self.client);

        let value = erc20.allowance(self.pubkey, spender_addy).call().await?;

        let deci = erc20.decimals().call().await?;
        let allow = format_units(value, deci)?;
        let allowance = allow.parse()?;

        Ok(allowance)
    }

    /// Returns the wallet's public Ethereum address as a hex string.
    pub fn get_pubkey(&self) -> Result<String, EvmWalletErrors> {
        Ok(self.pubkey.to_string())
    }

    /// Parses an ether amount in `f64` into wei `U256`.
    pub fn parse_native(&self, amount: f64) -> Result<U256, EvmWalletErrors> {
        let amt: U256 = parse_ether(&amount.to_string())?;

        Ok(amt)
    }

    /// Formats a wei `U256` into an ether `f64`.
    pub fn format_native(&self, amount: U256) -> Result<f64, EvmWalletErrors> {
        let amnt = format_ether(amount).parse()?;

        Ok(amnt)
    }

    /// Parses a token amount in `f64` using the token's on-chain decimal places into `U256`.
    pub async fn parse_token(&self, amount: f64, token: &str) -> Result<U256, EvmWalletErrors> {
        let token_addy = Address::from_str(token)?;
        let erc20 = ERC20::new(token_addy, &self.client);
        let deci = erc20.decimals().call().await?;

        let amt = parse_units(&amount.to_string(), deci)?.into();

        Ok(amt)
    }

    /// Formats a token balance in `U256` using the token's on-chain decimal places into `f64`.
    pub async fn format_token(&self, amount: U256, token: &str) -> Result<f64, EvmWalletErrors> {
        let token_addy = Address::from_str(token)?;
        let erc20 = ERC20::new(token_addy, &self.client);
        let deci = erc20.decimals().call().await?;

        let amt = format_units(amount, deci)?.parse()?;

        Ok(amt)
    }

    /// Transfers approximately the entire native balance to `to`, subtracting estimated gas fees.
    pub async fn close(&self, to: &str) -> Result<(), EvmWalletErrors> {
        let to_addy = Address::from_str(to)?;
        let bal = self.balance().await?;

        let wei = parse_ether(&(bal * 0.9).to_string())?;

        let fees = self.client.estimate_eip1559_fees().await?;

        let tx =
            TransactionRequest::default().with_from(self.pubkey).with_to(to_addy).with_value(wei);

        let gas = self.client.estimate_gas(tx).await?;

        let total_fees_wei = (gas as u128) * fees.max_fee_per_gas;
        let total_fees: f64 = format_ether(total_fees_wei).parse()?;

        // 2 percent higher fee buffer
        let _ = self.transfer(to, bal - (total_fees * 1.02)).await?;

        Ok(())
    }

    /// Fetches the wallet's native gas token (ETH) balance.
    pub async fn balance(&self) -> Result<f64, EvmWalletErrors> {
        let bal = self.client.get_balance(self.pubkey).await?;

        let fbal = format_ether(bal);

        Ok(fbal.parse()?)
    }

    /// Transfers native tokens (ETH) to a recipient address.
    pub async fn transfer(
        &self,
        to: &str,
        amount: f64,
    ) -> Result<TransactionReceipt, EvmWalletErrors> {
        let _signer = self.signer.as_ref().ok_or(GeneralWalletErrors::MissingSigner)?;
        let to_addy = Address::from_str(to)?;
        let wei = parse_ether(&amount.to_string())?;

        let tx =
            TransactionRequest::default().with_from(self.pubkey).with_to(to_addy).with_value(wei);

        let sig = self.client.send_transaction(tx).await?.get_receipt().await?;

        Ok(sig)
    }

    /// Fetches the wallet's balance of the specified ERC-20 token.
    pub async fn token_balance(&self, token: &str) -> Result<f64, EvmWalletErrors> {
        let token_addy = Address::from_str(token)?;

        // Instantiate the contract instance
        let erc20 = ERC20::new(token_addy, &self.client);

        // Fetch the token balance and decimals
        let balance = erc20.balanceOf(self.pubkey).call().await?;
        let deci = erc20.decimals().call().await?;

        let bal = format_units(balance, deci)?;

        Ok(bal.parse()?)
    }

    /// Transfers an ERC-20 token amount to a recipient address.
    pub async fn transfer_token(
        &self,
        token: &str,
        amount: f64,
        to: &str,
    ) -> Result<TransactionReceipt, EvmWalletErrors> {
        let _signer = self.signer.as_ref().ok_or(GeneralWalletErrors::MissingSigner)?;
        let to_addy = Address::from_str(to)?;
        let token_addy = Address::from_str(token)?;

        let erc20 = ERC20::new(token_addy, &self.client);
        let deci = erc20.decimals().call().await?;

        let amnt: Uint<256, 4> = parse_units(&amount.to_string(), deci)?.into();

        let sig = erc20.transfer(to_addy, amnt).send().await?.get_receipt().await?;

        Ok(sig)
    }

    /// Transfers the entire balance of an ERC-20 token to a recipient address.
    pub async fn transfer_all_tokens(&self, token: &str, to: &str) -> Result<(), EvmWalletErrors> {
        let amount = self.token_balance(token).await?;

        if amount != 0.0 {
            let _ = self.transfer_token(token, amount, to).await?;
        }

        Ok(())
    }

    /// Signs and sends a transaction request, returning its receipt.
    pub async fn sign_and_send(
        &self,
        txn: TransactionRequest,
    ) -> Result<TransactionReceipt, EvmWalletErrors> {
        let _signer = self.signer.as_ref().ok_or(GeneralWalletErrors::MissingSigner)?;
        let sig = self.client.send_transaction(txn).await?.get_receipt().await?;

        Ok(sig)
    }
}
