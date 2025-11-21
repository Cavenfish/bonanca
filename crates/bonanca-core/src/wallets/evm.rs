use alloy::{
    network::TransactionBuilder,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::{k256::ecdsa::SigningKey, local::LocalSigner},
    sol,
    transports::http::reqwest::Url,
};
use alloy_primitives::{
    Address, Uint,
    utils::{format_ether, format_units, parse_ether, parse_units},
};
use anyhow::Result;
use async_trait::async_trait;
use bonanca_keyvault::{decrypt_keyvault, hd_keys::ChildKey, read_keyvault};
use core::panic;
use std::{path::Path, str::FromStr};

use crate::api_lib::traits::SwapTransactionData;
use crate::wallets::traits::Wallet;

// ABI for smart contracts
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "src/wallets/ABI/ERC20.json"
}

pub struct EvmWallet {
    pub signer: Option<LocalSigner<SigningKey>>,
    pub rpc: Url,
    pub pubkey: Address,
}

#[async_trait]
impl Wallet for EvmWallet {
    fn get_pubkey(&self) -> Result<String> {
        Ok(self.pubkey.to_string())
    }

    fn parse_native_amount(&self, amount: f64) -> Result<u64> {
        let amt = (amount * 1e18) as u64;

        Ok(amt)
    }

    async fn parse_token_amount(&self, amount: f64, token: &str) -> Result<u64> {
        let token_addy = Address::from_str(token)?;
        let client = self.get_view_client();

        // Instantiate the contract instance
        let erc20 = ERC20::new(token_addy, client);
        let deci = erc20.decimals().call().await?;

        let amt = (amount * 10.0_f64.powi(deci.into())) as u64;

        Ok(amt)
    }

    async fn close(&self, to: &str) -> Result<()> {
        let to_addy = Address::from_str(to)?;
        let client = self.get_client();
        let bal = self.balance().await?;

        let wei = parse_ether(&(bal * 0.9).to_string())?;

        let fees = client.estimate_eip1559_fees().await?;

        let tx = TransactionRequest::default()
            .with_from(self.pubkey)
            .with_to(to_addy)
            .with_value(wei);

        let gas = client.estimate_gas(tx).await?;

        let total_fees_wei = (gas as u128) * fees.max_fee_per_gas;
        let total_fees: f64 = format_ether(total_fees_wei).parse()?;

        // 2 percent higher fee buffer
        let _ = self.transfer(to, bal - (total_fees * 1.02)).await?;

        Ok(())
    }

    async fn balance(&self) -> Result<f64> {
        let client = self.get_view_client();

        let bal = client.get_balance(self.pubkey).await?;

        let fbal = format_ether(bal);

        Ok(fbal.parse()?)
    }

    async fn transfer(&self, to: &str, amount: f64) -> Result<()> {
        let to_addy = Address::from_str(to)?;
        let client = self.get_client();

        let wei = parse_ether(&amount.to_string())?;

        // Build transaction
        let tx = TransactionRequest::default()
            .with_from(self.pubkey)
            .with_to(to_addy)
            .with_value(wei);

        // Send the transaction and wait for it to finish
        let _ = client.send_transaction(tx).await?.watch().await?;

        Ok(())
    }

    async fn token_balance(&self, token: &str) -> Result<f64> {
        let token_addy = Address::from_str(token)?;
        let client = self.get_view_client();

        // Instantiate the contract instance
        let erc20 = ERC20::new(token_addy, client);

        // Fetch the token balance and decimals
        let balance = erc20.balanceOf(self.pubkey).call().await?;
        let deci = erc20.decimals().call().await?;

        let bal = format_units(balance, deci)?;

        Ok(bal.parse()?)
    }

    async fn transfer_token(&self, token: &str, amount: f64, to: &str) -> Result<()> {
        let to_addy = Address::from_str(to)?;
        let token_addy = Address::from_str(token)?;
        let client = self.get_client();

        // Load token contract and get decimals
        let erc20 = ERC20::new(token_addy, client);
        let deci = erc20.decimals().call().await?;

        // Format amount to send
        let amnt: Uint<256, 4> = parse_units(&amount.to_string(), deci)?.into();

        // Send transaction
        let _ = erc20.transfer(to_addy, amnt).send().await?.watch().await?;

        Ok(())
    }

    async fn transfer_all_tokens(&self, token: &str, to: &str) -> Result<()> {
        let amount = self.token_balance(token).await?;

        if amount != 0.0 {
            let _ = self.transfer_token(token, amount, to).await?;
        }

        Ok(())
    }

    async fn check_swap(&self, token: &str, amount: f64, spender: Option<&str>) -> Result<bool> {
        let bal = self.token_balance(token).await?;
        if bal < amount {
            return Ok(false);
        };

        let allow = self.get_token_allowance(token, spender.unwrap()).await?;
        if allow < amount {
            let to_add = amount - allow;

            let _ = self
                .approve_token_spending(token, spender.unwrap(), to_add)
                .await?;
        };

        Ok(true)
    }

    async fn swap(&self, swap_data: SwapTransactionData) -> Result<()> {
        let client = self.get_client();
        let tx = match swap_data {
            SwapTransactionData::Evm(trans) => trans,
            _ => Err(anyhow::anyhow!("Swap API does not work on this chain"))?,
        };

        let _ = client.send_transaction(tx).await?.watch().await?;

        Ok(())
    }
}

impl EvmWallet {
    pub fn load(keyvault: &Path, rpc: &str, child: u32) -> Self {
        let hd_key = decrypt_keyvault(keyvault).expect("Failed to decrypt keyvault");
        let child_key = hd_key.get_child_key("EVM", child).unwrap();

        let signer = match child_key {
            ChildKey::Evm(sig) => sig,
            _ => panic!(),
        };

        let rpc_url = Url::parse(rpc).unwrap();
        let pubkey = signer.address();
        Self {
            signer: Some(signer),
            rpc: rpc_url,
            pubkey,
        }
    }

    pub fn view(keyvault: &Path, rpc: &str, child: u32) -> Self {
        let key_vault = read_keyvault(keyvault).unwrap();
        let evm_keys = key_vault
            .chain_keys
            .iter()
            .find(|k| k.chain == "EVM")
            .unwrap();
        let pubkey = evm_keys.public_keys.get(child as usize).unwrap();
        let rpc_url = Url::parse(rpc).unwrap();
        let addy = Address::from_str(pubkey).unwrap();
        Self {
            signer: None,
            rpc: rpc_url,
            pubkey: addy,
        }
    }

    // Builds the client (RPC connection)
    fn get_client(&self) -> impl Provider {
        if self.signer.is_none() {
            panic!()
        };

        let signer = self.signer.clone().unwrap();

        ProviderBuilder::new()
            .wallet(signer)
            .connect_http(self.rpc.clone())
    }

    fn get_view_client(&self) -> impl Provider {
        ProviderBuilder::new().connect_http(self.rpc.clone())
    }

    // Approve token for spending
    async fn approve_token_spending(&self, token: &str, spender: &str, amount: f64) -> Result<()> {
        let token_addy = Address::from_str(token)?;
        let spender_addy = Address::from_str(spender)?;
        let client = self.get_client();

        let erc20 = ERC20::new(token_addy, client);

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

    async fn get_token_allowance(&self, token: &str, spender: &str) -> Result<f64> {
        let token_addy = Address::from_str(token)?;
        let spender_addy = Address::from_str(spender)?;
        let client = self.get_client();

        let erc20 = ERC20::new(token_addy, client);

        let value = erc20.allowance(self.pubkey, spender_addy).call().await?;

        let deci = erc20.decimals().call().await?;
        let allow = format_units(value, deci)?;

        Ok(allow.parse()?)
    }
}
