use alloy::{
    primitives::{Address, U256},
    rpc::types::TransactionReceipt,
    sol,
};
use anyhow::Result;
use bonanca_api_lib::defi::morpho::{
    MorphoApi, user_data_query::UserDataQueryUserByAddressVaultPositions,
    vaults_v1_query::VaultsV1QueryVaultsItems,
};
use bonanca_wallets::wallets::evm::EvmWallet;
use std::str::FromStr;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    VaultV1,
    "src/evm/ABI/morpho_vault_v1.json"
}

pub struct MorphoVaultV1 {
    api: MorphoApi,
}

impl MorphoVaultV1 {
    pub fn new() -> Self {
        let api = MorphoApi::new();

        Self { api }
    }

    pub async fn get_user_data(
        &self,
        user: &str,
        chain_id: i64,
    ) -> Result<Vec<UserDataQueryUserByAddressVaultPositions>> {
        Ok(self
            .api
            .query_user_data(user, chain_id)
            .await?
            .vault_positions)
    }

    pub async fn get_token_vaults(
        &self,
        token_symbol: &str,
        chain_id: i64,
    ) -> Result<Vec<VaultsV1QueryVaultsItems>> {
        self.api.query_vaults_v1(token_symbol, chain_id).await
    }

    pub async fn supply(
        &self,
        wallet: &EvmWallet,
        vault_address: &str,
        amount: f64,
    ) -> Result<TransactionReceipt> {
        let addy = Address::from_str(vault_address).unwrap();
        let vault = VaultV1::new(addy, &wallet.client);
        let token = vault.asset().call().await?;
        let amnt = wallet
            .parse_token_amount(amount, &token.to_string())
            .await?;

        let sig = vault
            .deposit(U256::from(amnt), wallet.pubkey)
            .send()
            .await?
            .get_receipt()
            .await?;

        Ok(sig)
    }

    pub async fn withdraw(
        &self,
        wallet: &EvmWallet,
        vault_address: &str,
        amount: f64,
    ) -> Result<TransactionReceipt> {
        let addy = Address::from_str(vault_address).unwrap();
        let vault = VaultV1::new(addy, &wallet.client);
        let token = vault.asset().call().await?;
        let amnt = wallet
            .parse_token_amount(amount, &token.to_string())
            .await?;

        let sig = vault
            .withdraw(U256::from(amnt), wallet.pubkey, wallet.pubkey)
            .send()
            .await?
            .get_receipt()
            .await?;

        Ok(sig)
    }
}
