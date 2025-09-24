use alloy::providers::fillers::{FillProvider, JoinFill};
use alloy::signers::k256::Secp256k1;
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::transports::http::reqwest::Url;
use alloy::{
    providers::{Provider, ProviderBuilder},
    signers::local::LocalSigner,
    sol,
};
use alloy_primitives::utils::format_ether;
use alloy_primitives::{Address, address};
use anyhow::Result;
use std::path::PathBuf;

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
    pub fn new(keystore: PathBuf, rpc: &str) -> Self {
        let signer = LocalSigner::decrypt_keystore(&keystore, "test").unwrap();
        let rpc_url = Url::parse(&rpc).unwrap();
        let pubkey = signer.address();
        Self {
            signer: signer,
            rpc: rpc_url,
            pubkey: pubkey,
        }
    }

    pub async fn balance(&self) -> Result<String> {
        let wallet = ProviderBuilder::new()
            .wallet(self.signer.clone())
            .connect_http(self.rpc.clone());

        let bal = wallet.get_balance(self.pubkey).await?;

        let fbal = format_ether(bal);

        Ok(fbal)
    }

    pub async fn token_balance(&self, token: Address) -> Result<String> {
        let wallet = ProviderBuilder::new()
            .wallet(self.signer.clone())
            .connect_http(self.rpc.clone());

        // Instantiate the contract instance.
        let erc20 = ERC20::new(token, wallet);

        // Fetch the balance of WETH for a given address.
        let balance = erc20.balanceOf(self.pubkey).call().await?;

        let bal = format_ether(balance);

        Ok(bal)
    }
}
