use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::fmt;

pub struct KaminoApi {
    base_url: String,
}

impl KaminoApi {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.kamino.finance".to_string(),
        }
    }

    pub async fn get_all_kvaults(&self) -> Result<Vec<KVaultInfo>> {
        let client = Client::new();
        let url = format!("{}/kvaults/vaults", &self.base_url);

        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<Vec<KVaultInfo>>()
            .await?;

        Ok(resp)
    }

    pub async fn get_user_data(&self, user: &str) -> Result<Vec<KVaultPosition>> {
        let client = Client::new();
        let url = format!("{}/kvaults/users/{}/positions", &self.base_url, user);

        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<Vec<KVaultPosition>>()
            .await?;

        Ok(resp)
    }
}

#[derive(Debug, Deserialize)]
pub struct KVaultInfo {
    pub address: String,
    pub state: VaultState,
    #[serde(rename = "programId")]
    pub program_id: String,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationStrategy {
    pub reserve: String,
    pub ctoken_vault: String,
    pub target_allocation_weight: u32,
    pub token_allocation_cap: String,
    pub ctoken_vault_bump: u8,
    pub ctoken_allocation: String,
    pub last_invest_slot: String,
    pub token_target_allocation: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KVaultPosition {
    pub vault_address: String,
    pub staked_shares: f64,
    pub unstaked_shares: f64,
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
