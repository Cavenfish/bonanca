use alloy::{
    network::TransactionBuilder,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::{k256::ecdsa::SigningKey, local::LocalSigner},
    sol,
    transports::http::reqwest::Url,
};
use alloy_primitives::{
    Address, U256,
    utils::{format_ether, format_units, parse_ether, parse_units},
};
use anyhow::Result;
use std::path::PathBuf;

// ABI for smart contracts
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "src/wallets/ABI/ERC20.json"
}

pub struct EvmWallet {
    pub signer: LocalSigner<SigningKey>,
    pub rpc: Url,
    pub pubkey: Address,
}

impl EvmWallet {
    pub fn from(keystore: PathBuf, rpc: &str) -> Self {
        let signer = LocalSigner::decrypt_keystore(&keystore, "test").unwrap();
        let rpc_url = Url::parse(&rpc).unwrap();
        let pubkey = signer.address();
        Self {
            signer: signer,
            rpc: rpc_url,
            pubkey: pubkey,
        }
    }

    // Builds the client (RPC connection)
    fn get_client(&self) -> impl Provider {
        ProviderBuilder::new()
            .wallet(self.signer.clone())
            .connect_http(self.rpc.clone())
    }

    pub async fn balance(&self) -> Result<String> {
        let client = self.get_client();

        let bal = client.get_balance(self.pubkey).await?;

        let fbal = format_ether(bal);

        Ok(fbal)
    }

    pub async fn token_balance(&self, token: Address) -> Result<String> {
        let client = self.get_client();

        // Instantiate the contract instance
        let erc20 = ERC20::new(token, client);

        // Fetch the token balance and decimals
        let balance = erc20.balanceOf(self.pubkey).call().await?;
        let deci = erc20.decimals().call().await?;

        let bal = format_units(balance, deci)?;

        Ok(bal)
    }

    pub async fn transfer(&self, amount: f64, to: Address) -> Result<()> {
        let client = self.get_client();

        let wei = parse_ether(&amount.to_string())?;

        // Build transaction
        let tx = TransactionRequest::default()
            .with_from(self.pubkey)
            .with_to(to)
            .with_value(wei);

        // Send the transaction and wait for it to finish
        let _ = client.send_transaction(tx).await?.watch().await?;

        Ok(())
    }

    pub async fn transfer_token(&self, amount: f64, to: Address, token: Address) -> Result<()> {
        let client = self.get_client();

        // Load token contract and get decimals
        let erc20 = ERC20::new(token, client);
        let deci = erc20.decimals().call().await?;

        // Format amount to send
        let amnt = parse_units(&amount.to_string(), deci)?.into();

        // Send transaction
        let _ = erc20.transfer(to, amnt).send().await?.watch().await?;

        Ok(())
    }
}
