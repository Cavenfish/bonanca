use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use std::fmt;

pub struct KaminoApi {
    base_url: String,
    client: Client,
}

impl KaminoApi {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.kamino.finance".to_string(),
            client: Client::new(),
        }
    }

    pub async fn get_all_kvaults(&self) -> Result<Vec<KVaultInfo>> {
        let url = format!("{}/kvaults/vaults", &self.base_url);

        let resp = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<Vec<KVaultInfo>>()
            .await?;

        Ok(resp)
    }

    pub async fn get_vault_metrics(&self, vault: &str) -> Result<VaultMetrics> {
        let url = format!("{}/kvaults/vaults/{}/metrics", &self.base_url, vault);

        let resp = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<VaultMetrics>()
            .await?;

        Ok(resp)
    }

    // pub async fn get_token_rates(&self, token: &str) -> Result<Vec<LendingRate>> {
    //     let mut rates: Vec<LendingRate> = Vec::new();

    //     let vaults = self.get_all_kvaults().await?;

    //     for vault in vaults.iter().filter(|v| v.state.name.contains(token)) {
    //         let metrics = self.get_vault_metrics(&vault.address).await?;

    //         rates.push(LendingRate {
    //             apy: metrics.apy.parse()?,
    //             protocol: "Kamino".to_string(),
    //             token: token.to_string(),
    //             vault_name: vault.state.name.clone(),
    //         });
    //     }

    //     Ok(rates)
    // }

    pub async fn get_user_data(&self, user: &str) -> Result<Vec<KVaultPosition>> {
        let url = format!("{}/kvaults/users/{}/positions", &self.base_url, user);

        let resp = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<Vec<KVaultPosition>>()
            .await?;

        Ok(resp)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct KVaultInfo {
    pub address: String,
    pub state: VaultState,
    #[serde(rename = "programId")]
    pub program_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultState {
    pub vault_admin_authority: String,
    pub base_vault_authority: String,
    pub base_vault_authority_bump: u8,
    pub token_mint: String,
    pub token_mint_decimals: u8,
    pub token_vault: String,
    pub token_program: String,
    pub shares_mint: String,
    pub shares_mint_decimals: u8,
    pub token_available: String,
    pub shares_issued: String,
    pub available_crank_funds: String,
    pub performance_fee_bps: u32,
    pub management_fee_bps: u32,
    pub last_fee_charge_timestamp: u64,
    pub prev_aum: String,
    pub pending_fees: String,
    pub vault_allocation_strategy: Vec<AllocationStrategy>,
    pub min_deposit_amount: String,
    pub min_withdraw_amount: String,
    pub min_invest_amount: String,
    pub min_invest_delay_slots: u64,
    pub crank_fund_fee_per_reserve: String,
    pub pending_admin: String,
    pub cumulative_earned_interest: String,
    pub cumulative_mgmt_fees: String,
    pub cumulative_perf_fees: String,
    pub name: String,
    pub vault_lookup_table: String,
    pub vault_farm: String,
    pub creation_timestamp: u64,
    pub allocation_admin: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationStrategy {
    pub reserve: String,
    pub ctoken_vault: String,
    pub target_allocation_weight: u64,
    pub token_allocation_cap: String,
    pub ctoken_vault_bump: u8,
    pub ctoken_allocation: String,
    pub last_invest_slot: String,
    pub token_target_allocation: String,
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KVaultPosition {
    pub vault_address: String,
    #[serde_as(as = "DisplayFromStr")]
    pub staked_shares: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub unstaked_shares: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub total_shares: f64,
}

impl fmt::Display for KVaultPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Vault Address: {}", self.vault_address)?;
        writeln!(f, "Staked Shares: {}", self.staked_shares)?;
        writeln!(f, "Unstaked Shares: {}", self.unstaked_shares)?;
        writeln!(f, "Total Shares: {}", self.total_shares)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultMetrics {
    pub apy_7d: String,
    pub apy_24h: String,
    pub apy_30d: String,
    pub apy_90d: String,
    pub apy_180d: String,
    pub apy_365d: String,
    pub token_price: String,
    pub sol_price: String,
    pub tokens_available: String,
    pub tokens_available_usd: String,
    pub tokens_invested: String,
    pub tokens_invested_usd: String,
    pub share_price: String,
    pub tokens_per_share: String,
    pub apy: String,
    pub apy_theoretical: String,
    pub apy_actual: String,
    pub apy_farm_rewards: String,
    pub apy_incentives: String,
    pub apy_reserves_incentives: String,
    pub number_of_holders: u64,
    pub shares_issued: String,
    pub cumulative_interest_earned: String,
    pub cumulative_interest_earned_usd: String,
    pub cumulative_interest_earned_sol: String,
    pub interest_earned_per_second: String,
    pub interest_earned_per_second_usd: String,
    pub interest_earned_per_second_sol: String,
    pub cumulative_performance_fees: String,
    pub cumulative_performance_fees_usd: String,
    pub cumulative_performance_fees_sol: String,
    pub cumulative_management_fees: String,
    pub cumulative_management_fees_usd: String,
    pub cumulative_management_fees_sol: String,
}
