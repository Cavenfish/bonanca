use anchor_client::{
    Client, Cluster,
    solana_sdk::{
        commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair,
        signer::SeedDerivable,
    },
};
use anchor_lang::prelude::*;
use anyhow::Result;
use bonanca_api_lib::defi::kamino::{KVaultInfo, KVaultPosition, KaminoApi};
use bonanca_wallets::wallets::solana::SolWallet;
use std::{rc::Rc, str::FromStr};

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

    pub async fn get_vaults(&self) -> Result<Vec<KVaultInfo>> {
        self.api.get_all_kvaults().await
    }

    pub async fn get_vault_data(&self, name: &str) -> Result<KVaultInfo> {
        let vaults = self.api.get_all_kvaults().await?;

        let vault = vaults.into_iter().find(|v| v.state.name == name).unwrap();

        Ok(vault)
    }

    pub async fn get_user_data(&self, pubkey: &str) -> Result<Vec<KVaultPosition>> {
        self.api.get_user_data(&pubkey).await
    }

    pub async fn get_token_vaults(&self, token: &str) -> Result<Vec<KVaultInfo>> {
        let vaults = self.api.get_all_kvaults().await?;

        let token_vaults: Vec<KVaultInfo> = vaults
            .into_iter()
            .filter(|v| v.state.token_mint == token)
            .collect();

        Ok(token_vaults)
    }

    pub async fn supply(
        &self,
        wallet: &SolWallet,
        vault_data: &KVaultInfo,
        amount: f64,
    ) -> Result<()> {
        // These two conversion are because Anchor and solana_sdk use different versions
        let payer = Keypair::from_seed(wallet.key_pair.as_ref().unwrap().secret_bytes())
            .expect("Couldn't conver keypair types");
        let user = Pubkey::from_str(&wallet.pubkey.to_string())?;

        let provider = Client::new_with_options(
            Cluster::Localnet,
            Rc::new(payer),
            CommitmentConfig::confirmed(),
        );

        let program = provider.program(kvault::ID)?;

        let token_mint = Pubkey::from_str_const(&vault_data.state.token_mint);
        let token_vault = Pubkey::from_str_const(&vault_data.state.token_vault);
        let base_vault_authority = Pubkey::from_str_const(&vault_data.state.base_vault_authority);
        let shares_mint = Pubkey::from_str_const(&vault_data.state.shares_mint);
        let token_program = Pubkey::from_str_const(&vault_data.state.token_program);
        let vault_state = Pubkey::from_str_const(&vault_data.address);

        let user_token_ata = get_ata(&user, &token_mint);
        let user_shares_ata = get_ata(&user, &shares_mint);

        let event_authority = get_event_authority(&program.id());

        let amnt = wallet
            .parse_token_amount(amount, &vault_data.state.token_mint)
            .await?;

        let supply_ix = program
            .request()
            .accounts(accounts::Deposit {
                user,
                vault_state,
                token_vault,
                token_mint,
                base_vault_authority,
                shares_mint,
                user_token_ata,
                user_shares_ata,
                klend_program: KLEND_ID,
                token_program,
                shares_token_program: TOKEN_ID,
                event_authority,
                program: program.id(),
            })
            .args(args::Deposit { max_amount: amnt })
            .instructions()?
            .remove(0);

        let _ = program.request().instruction(supply_ix).send().await?;

        Ok(())
    }

    pub async fn withdraw(
        &self,
        wallet: &SolWallet,
        vault_data: &KVaultInfo,
        amount: f64,
    ) -> Result<()> {
        // These two conversion are because Anchor and solana_sdk use different versions
        let payer = Keypair::from_seed(wallet.key_pair.as_ref().unwrap().secret_bytes())
            .expect("Couldn't conver keypair types");
        let user = Pubkey::from_str(&wallet.pubkey.to_string())?;

        let provider = Client::new_with_options(
            Cluster::Localnet,
            Rc::new(payer),
            CommitmentConfig::confirmed(),
        );

        let program = provider.program(kvault::ID)?;

        let token_mint = Pubkey::from_str_const(&vault_data.state.token_mint);
        let token_vault = Pubkey::from_str_const(&vault_data.state.token_vault);
        let base_vault_authority = Pubkey::from_str_const(&vault_data.state.base_vault_authority);
        let shares_mint = Pubkey::from_str_const(&vault_data.state.shares_mint);
        let token_program = Pubkey::from_str_const(&vault_data.state.token_program);
        let vault_state = Pubkey::from_str_const(&vault_data.address);

        let user_token_ata = get_ata(&user, &token_mint);
        let user_shares_ata = get_ata(&user, &shares_mint);

        let event_authority = get_event_authority(&program.id());

        let amnt = wallet
            .parse_token_amount(amount, &vault_data.state.token_mint)
            .await?;

        // This address was taken from Leo AI chat
        // Need to find more reliable source of information
        let global_config = Pubkey::from_str_const("GcJ95j99v76X95Y17o29149v896584792K4YvXK3152v");

        let withdraw_from_available = accounts::WithdrawFromAvailable {
            user,
            vault_state,
            token_vault,
            token_mint,
            base_vault_authority,
            shares_mint,
            user_token_ata,
            user_shares_ata,
            klend_program: KLEND_ID,
            token_program,
            shares_token_program: TOKEN_ID,
            event_authority,
            program: program.id(),
            global_config,
        };

        let supply_ix = program
            .request()
            .accounts(accounts::Withdraw {
                withdraw_from_available,
                event_authority,
                program: program.id(),
            })
            .args(args::Deposit { max_amount: amnt })
            .instructions()?
            .remove(0);

        let _ = program.request().instruction(supply_ix).send().await?;

        Ok(())
    }
}
