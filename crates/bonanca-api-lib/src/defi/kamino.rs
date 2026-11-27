//! Kamino Vaults API client and vault/position types.

use std::fmt;

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

/// Client for the Kamino Vaults REST API.
pub struct KaminoApi {
    base_url: String,
    client: Client,
}

impl KaminoApi {
    /// Creates a new [`KaminoApi`] client pointing at `https://api.kamino.finance`.
    pub fn new() -> Self {
        Self { base_url: "https://api.kamino.finance".to_string(), client: Client::new() }
    }

    /// Fetches information about all Kamino Vaults.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into `Vec<KVaultInfo>`.
    pub async fn get_all_kvaults(&self) -> Result<Vec<KVaultInfo>> {
        let url = format!("{}/kvaults/vaults", &self.base_url);

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<Vec<KVaultInfo>>()
            .await?;

        Ok(resp)
    }

    /// Fetches metrics for the vault at address `vault`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into [`VaultMetrics`].
    pub async fn get_vault_metrics(&self, vault: &str) -> Result<VaultMetrics> {
        let url = format!("{}/kvaults/vaults/{}/metrics", &self.base_url, vault);

        let resp = self
            .client
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

    /// Fetches all Kamino Vault positions for `user`.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or if the response body cannot be deserialized
    /// into `Vec<KVaultPosition>`.
    pub async fn get_user_data(&self, user: &str) -> Result<Vec<KVaultPosition>> {
        let url = format!("{}/kvaults/users/{}/positions", &self.base_url, user);

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<Vec<KVaultPosition>>()
            .await?;

        Ok(resp)
    }
}

/// Kamino Vault information.
#[derive(Clone, Debug, Deserialize)]
pub struct KVaultInfo {
    /// Vault address.
    pub address: String,
    /// On-chain vault state.
    pub state: VaultState,
    /// Vault program ID.
    #[serde(rename = "programId")]
    pub program_id: String,
}

/// On-chain state of a Kamino Vault.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultState {
    /// Authority allowed to administer the vault.
    pub vault_admin_authority: String,
    /// Authority over the base token vault.
    pub base_vault_authority: String,
    /// Bump seed for `base_vault_authority`.
    pub base_vault_authority_bump: u8,
    /// Mint address of the deposited token.
    pub token_mint: String,
    /// Decimals of the deposited token.
    pub token_mint_decimals: u8,
    /// Token account holding deposited tokens.
    pub token_vault: String,
    /// Token program used by the deposited token.
    pub token_program: String,
    /// Mint address of the vault shares token.
    pub shares_mint: String,
    /// Decimals of the shares token.
    pub shares_mint_decimals: u8,
    /// Amount of tokens available in the vault.
    pub token_available: String,
    /// Total shares issued.
    pub shares_issued: String,
    /// Funds available for crank operations.
    pub available_crank_funds: String,
    /// Performance fee in basis points.
    pub performance_fee_bps: u32,
    /// Management fee in basis points.
    pub management_fee_bps: u32,
    /// Timestamp of the last fee charge.
    pub last_fee_charge_timestamp: u64,
    /// Assets under management before the last update.
    pub prev_aum: String,
    /// Fees pending collection.
    pub pending_fees: String,
    /// Vault allocation strategy entries.
    pub vault_allocation_strategy: Vec<AllocationStrategy>,
    /// Minimum deposit amount.
    pub min_deposit_amount: String,
    /// Minimum withdrawal amount.
    pub min_withdraw_amount: String,
    /// Minimum investment amount.
    pub min_invest_amount: String,
    /// Minimum delay before investing, in slots.
    pub min_invest_delay_slots: u64,
    /// Crank fee per reserve.
    pub crank_fund_fee_per_reserve: String,
    /// Pending admin address, if any.
    pub pending_admin: String,
    /// Cumulative interest earned.
    pub cumulative_earned_interest: String,
    /// Cumulative management fees charged.
    pub cumulative_mgmt_fees: String,
    /// Cumulative performance fees charged.
    pub cumulative_perf_fees: String,
    /// Vault name.
    pub name: String,
    /// Address lookup table for the vault.
    pub vault_lookup_table: String,
    /// Farm address, if any.
    pub vault_farm: String,
    /// Vault creation timestamp.
    pub creation_timestamp: u64,
    /// Authority allowed to administer allocations.
    pub allocation_admin: String,
}

/// Vault allocation strategy entry for a reserve.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationStrategy {
    /// Reserve address.
    pub reserve: String,
    /// cToken vault address.
    pub ctoken_vault: String,
    /// Target allocation weight.
    pub target_allocation_weight: u64,
    /// Cap on token allocation.
    pub token_allocation_cap: String,
    /// Bump seed for `ctoken_vault`.
    pub ctoken_vault_bump: u8,
    /// Current cToken allocation.
    pub ctoken_allocation: String,
    /// Slot of the last investment.
    pub last_invest_slot: String,
    /// Target token allocation.
    pub token_target_allocation: String,
}

/// A user's position in a Kamino Vault.
#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KVaultPosition {
    /// Vault address.
    pub vault_address: String,
    /// Shares currently staked.
    #[serde_as(as = "DisplayFromStr")]
    pub staked_shares: f64,
    /// Shares not staked.
    #[serde_as(as = "DisplayFromStr")]
    pub unstaked_shares: f64,
    /// Total shares owned.
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

/// Metrics for a Kamino Vault.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultMetrics {
    /// APY over the last 7 days.
    pub apy_7d: String,
    /// APY over the last 24 hours.
    pub apy_24h: String,
    /// APY over the last 30 days.
    pub apy_30d: String,
    /// APY over the last 90 days.
    pub apy_90d: String,
    /// APY over the last 180 days.
    pub apy_180d: String,
    /// APY over the last 365 days.
    pub apy_365d: String,
    /// Current price of the deposited token.
    pub token_price: String,
    /// Current price of SOL.
    pub sol_price: String,
    /// Tokens available in the vault.
    pub tokens_available: String,
    /// Tokens available, in USD.
    pub tokens_available_usd: String,
    /// Tokens invested in reserves.
    pub tokens_invested: String,
    /// Tokens invested, in USD.
    pub tokens_invested_usd: String,
    /// Price per share.
    pub share_price: String,
    /// Tokens per share.
    pub tokens_per_share: String,
    /// Current APY.
    pub apy: String,
    /// Theoretical APY.
    pub apy_theoretical: String,
    /// Actual APY.
    pub apy_actual: String,
    /// APY from farm rewards.
    pub apy_farm_rewards: String,
    /// APY from incentives.
    pub apy_incentives: String,
    /// APY from reserve incentives.
    pub apy_reserves_incentives: String,
    /// Number of share holders.
    pub number_of_holders: u64,
    /// Total shares issued.
    pub shares_issued: String,
    /// Cumulative interest earned.
    pub cumulative_interest_earned: String,
    /// Cumulative interest earned, in USD.
    pub cumulative_interest_earned_usd: String,
    /// Cumulative interest earned, in SOL.
    pub cumulative_interest_earned_sol: String,
    /// Interest earned per second.
    pub interest_earned_per_second: String,
    /// Interest earned per second, in USD.
    pub interest_earned_per_second_usd: String,
    /// Interest earned per second, in SOL.
    pub interest_earned_per_second_sol: String,
    /// Cumulative performance fees charged.
    pub cumulative_performance_fees: String,
    /// Cumulative performance fees charged, in USD.
    pub cumulative_performance_fees_usd: String,
    /// Cumulative performance fees charged, in SOL.
    pub cumulative_performance_fees_sol: String,
    /// Cumulative management fees charged.
    pub cumulative_management_fees: String,
    /// Cumulative management fees charged, in USD.
    pub cumulative_management_fees_usd: String,
    /// Cumulative management fees charged, in SOL.
    pub cumulative_management_fees_sol: String,
}
