use alloy::{
    network::TransactionBuilder,
    providers::{DynProvider, Provider, ProviderBuilder},
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::{Signer, k256::ecdsa::SigningKey, local::LocalSigner},
    sol,
    transports::http::reqwest::Url,
};
use alloy_primitives::{
    Address, FixedBytes, Signature, Uint,
    utils::{format_ether, format_units, parse_ether, parse_units},
};
use anyhow::Result;
use bonanca_keyvault::{hd_keys::ChildKey, keyvault::KeyVault};
use core::panic;
use std::{path::Path, str::FromStr};

// ABI for smart contracts
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "src/wallets/ABI/ERC20.json"
}

pub struct EvmWallet {
    pub signer: Option<LocalSigner<SigningKey>>,
    pub client: DynProvider,
    pub pubkey: Address,
}

impl EvmWallet {
    pub fn load(keyvault: &Path, rpc: &str, child: u32) -> Self {
        let key_vault = KeyVault::load(keyvault);
        let child_key = key_vault.get_child_key("EVM", child).unwrap();

        let signer = match child_key {
            ChildKey::Evm(sig) => sig,
            _ => panic!(),
        };

        let rpc_url = Url::parse(rpc).unwrap();
        let pubkey = signer.address();

        let client: DynProvider = ProviderBuilder::new()
            .wallet(signer.clone())
            .connect_http(rpc_url)
            .erased();

        Self {
            signer: Some(signer),
            client,
            pubkey,
        }
    }

    pub fn view(keyvault: &Path, rpc: &str, child: u32) -> Self {
        let key_vault = KeyVault::load(keyvault);
        let evm_keys = key_vault.chain_keys.get("EVM").unwrap();
        let pubkey = evm_keys.get(child as usize).unwrap();
        let rpc_url = Url::parse(rpc).unwrap();
        let addy = Address::from_str(pubkey).unwrap();
        let client: DynProvider = ProviderBuilder::new().connect_http(rpc_url).erased();

        Self {
            signer: None,
            client,
            pubkey: addy,
        }
    }

    pub async fn sign_hash(&self, hash: &FixedBytes<32>) -> Result<Signature> {
        let sig = self.signer.as_ref().unwrap().sign_hash(&hash).await?;

        Ok(sig)
    }

    pub async fn approve_token_spending(
        &self,
        token: &str,
        spender: &str,
        amount: f64,
    ) -> Result<()> {
        let token_addy = Address::from_str(token)?;
        let spender_addy = Address::from_str(spender)?;

        let erc20 = ERC20::new(token_addy, &self.client);

        let decimals = erc20.decimals().call().await?;
        let value: Uint<256, 4> = parse_units(&amount.to_string(), decimals)?.into();

        let _ = erc20
            .approve(spender_addy, value)
            .send()
            .await?
            .watch()
            .await?;

        Ok(())
    }

    pub async fn get_token_allowance(&self, token: &str, spender: &str) -> Result<f64> {
        let token_addy = Address::from_str(token)?;
        let spender_addy = Address::from_str(spender)?;

        let erc20 = ERC20::new(token_addy, &self.client);

        let value = erc20.allowance(self.pubkey, spender_addy).call().await?;

        let deci = erc20.decimals().call().await?;
        let allow = format_units(value, deci)?;

        Ok(allow.parse()?)
    }

    pub fn get_pubkey(&self) -> Result<String> {
        Ok(self.pubkey.to_string())
    }

    pub fn format_native(&self, amount: f64) -> Result<u64> {
        let amt = (amount * 1e18) as u64;

        Ok(amt)
    }

    pub fn parse_native(&self, amount: u64) -> Result<f64> {
        Ok((amount as f64) / 1.0e18)
    }

    pub async fn format_token(&self, amount: f64, token: &str) -> Result<u64> {
        let token_addy = Address::from_str(token)?;
        let erc20 = ERC20::new(token_addy, &self.client);
        let deci = erc20.decimals().call().await?;

        let amt = (amount * 10.0_f64.powi(deci.into())) as u64;

        Ok(amt)
    }

    pub async fn parse_token(&self, amount: u64, token: &str) -> Result<f64> {
        let token_addy = Address::from_str(token)?;
        let erc20 = ERC20::new(token_addy, &self.client);
        let deci = erc20.decimals().call().await?;

        Ok((amount as f64) / 10.0_f64.powi(deci.into()))
    }

    pub async fn close(&self, to: &str) -> Result<()> {
        let to_addy = Address::from_str(to)?;
        let bal = self.balance().await?;

        let wei = parse_ether(&(bal * 0.9).to_string())?;

        let fees = self.client.estimate_eip1559_fees().await?;

        let tx = TransactionRequest::default()
            .with_from(self.pubkey)
            .with_to(to_addy)
            .with_value(wei);

        let gas = self.client.estimate_gas(tx).await?;

        let total_fees_wei = (gas as u128) * fees.max_fee_per_gas;
        let total_fees: f64 = format_ether(total_fees_wei).parse()?;

        // 2 percent higher fee buffer
        let _ = self.transfer(to, bal - (total_fees * 1.02)).await?;

        Ok(())
    }

    pub async fn balance(&self) -> Result<f64> {
        let bal = self.client.get_balance(self.pubkey).await?;

        let fbal = format_ether(bal);

        Ok(fbal.parse()?)
    }

    pub async fn transfer(&self, to: &str, amount: f64) -> Result<TransactionReceipt> {
        let to_addy = Address::from_str(to)?;
        let wei = parse_ether(&amount.to_string())?;

        let tx = TransactionRequest::default()
            .with_from(self.pubkey)
            .with_to(to_addy)
            .with_value(wei);

        let sig = self
            .client
            .send_transaction(tx)
            .await?
            .get_receipt()
            .await?;

        Ok(sig)
    }

    pub async fn token_balance(&self, token: &str) -> Result<f64> {
        let token_addy = Address::from_str(token)?;

        // Instantiate the contract instance
        let erc20 = ERC20::new(token_addy, &self.client);

        // Fetch the token balance and decimals
        let balance = erc20.balanceOf(self.pubkey).call().await?;
        let deci = erc20.decimals().call().await?;

        let bal = format_units(balance, deci)?;

        Ok(bal.parse()?)
    }

    pub async fn transfer_token(
        &self,
        token: &str,
        amount: f64,
        to: &str,
    ) -> Result<TransactionReceipt> {
        let to_addy = Address::from_str(to)?;
        let token_addy = Address::from_str(token)?;

        let erc20 = ERC20::new(token_addy, &self.client);
        let deci = erc20.decimals().call().await?;

        let amnt: Uint<256, 4> = parse_units(&amount.to_string(), deci)?.into();

        let sig = erc20
            .transfer(to_addy, amnt)
            .send()
            .await?
            .get_receipt()
            .await?;

        Ok(sig)
    }

    pub async fn transfer_all_tokens(&self, token: &str, to: &str) -> Result<()> {
        let amount = self.token_balance(token).await?;

        if amount != 0.0 {
            let _ = self.transfer_token(token, amount, to).await?;
        }

        Ok(())
    }

    pub async fn sign_and_send(&self, txn: TransactionRequest) -> Result<TransactionReceipt> {
        let sig = self
            .client
            .send_transaction(txn)
            .await?
            .get_receipt()
            .await?;

        Ok(sig)
    }
}
