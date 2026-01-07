use anchor_client::{
    Client, Cluster,
    solana_sdk::{
        commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair, signer::Signer,
    },
};
use anchor_lang::prelude::*;
use anyhow::Result;
use bonanca_api_lib::defi::kamino::{KVaultInfo, KaminoApi};
use std::{path::Path, sync::Arc};

const TOKEN_ID: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const SYSTEM_ID: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
const KLEND_ID: Pubkey = Pubkey::from_str_const("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD");

fn get_ata(user: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[&user.to_bytes(), &TOKEN_ID.to_bytes(), &mint.to_bytes()],
        &SYSTEM_ID,
    )
    .0
}

fn get_event_authority(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&["__event_authority".as_bytes()], &program_id).0
}

declare_program!(kvault);

use kvault::{client::accounts, client::args};

pub struct Kamino {
    api: KaminoApi,
}

impl Kamino {
    pub fn new() -> Self {
        let api = KaminoApi::new();
        Self { api }
    }

    async fn get_vault_data(&self, name: &str) -> Result<KVaultInfo> {
        let vaults = self.api.get_all_kvaults().await?;

        let vault = vaults.iter().find(|v| v.state.name == name).unwrap();

        Ok(vault.clone())
    }

    async fn get_pools(&self) -> Result<()> {
        let pools = self.api.get_all_kvaults().await?;

        for pool in pools.iter() {
            println!("Pool Address: {}", pool.address);
            println!("Token Available: {}", pool.state.token_available);
            println!("Min Deposit Amount: {}", pool.state.min_deposit_amount);
            println!("Performance Fee: {} bps", pool.state.performance_fee_bps);
            println!("Management Fee: {} bps", pool.state.management_fee_bps);
        }

        Ok(())
    }

    async fn get_user_data(&self, pubkey: &str) -> Result<()> {
        let data = self.api.get_user_data(&pubkey).await?;

        data.iter().for_each(|f| println!("{}", f));

        Ok(())
    }

    async fn get_token_pools(&self, token: &str) -> Result<()> {
        let vaults = self.api.get_all_kvaults().await?;

        let token_vaults: Vec<&KVaultInfo> = vaults
            .iter()
            .filter(|v| v.state.token_mint == token)
            .collect();

        token_vaults
            .iter()
            .for_each(|v| println!("Vault name: {}", v.state.name));

        Ok(())
    }

    async fn supply(&self, token: &str, amount: u64) -> Result<()> {
        // let user = self.user.pubkey();
        // let token_mint = Pubkey::from_str_const(token);
        // let client = self.get_client();

        // let program = client.program(kvault::ID)?;

        // let vault_data = self.get_vault_data().await?;

        // let token_vault = Pubkey::from_str_const(&vault_data.state.token_vault);
        // let base_vault_authority = Pubkey::from_str_const(&vault_data.state.base_vault_authority);
        // let shares_mint = Pubkey::from_str_const(&vault_data.state.shares_mint);
        // let token_program = Pubkey::from_str_const(&vault_data.state.token_program);
        // let vault_state = Pubkey::from_str_const(&vault_data.address);

        // let user_token_ata = get_ata(&user, &token_mint);
        // let user_shares_ata = get_ata(&user, &shares_mint);

        // let event_authority = get_event_authority(&program.id());

        // let supply_ix = program
        //     .request()
        //     .accounts(accounts::Deposit {
        //         user,
        //         vault_state,
        //         token_vault,
        //         token_mint,
        //         base_vault_authority,
        //         shares_mint,
        //         user_token_ata,
        //         user_shares_ata,
        //         klend_program: KLEND_ID,
        //         token_program,
        //         shares_token_program: TOKEN_ID,
        //         event_authority,
        //         program: program.id(),
        //     })
        //     .args(args::Deposit { max_amount: amount })
        //     .instructions()?
        //     .remove(0);

        // let _ = program.request().instruction(supply_ix).send().await?;

        Ok(())
    }

    async fn borrow(&self, token: &str, amount: u64) -> Result<()> {
        Ok(())
    }

    async fn repay(&self, token: &str, amount: u64) -> Result<()> {
        Ok(())
    }

    async fn withdraw(&self, token: &str, amount: u64) -> Result<()> {
        Ok(())
    }
}
