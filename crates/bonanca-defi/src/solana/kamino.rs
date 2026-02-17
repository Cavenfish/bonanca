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

const SYSVAR: Pubkey = Pubkey::from_str_const("Sysvar1nstructions1111111111111111111111111");
const TOKEN_ID: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const KLEND_ID: Pubkey = Pubkey::from_str_const("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD");

fn get_event_authority(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&["__event_authority".as_bytes()], &program_id).0
}

declare_program!(kvault);

use kvault::{
    accounts::{Reserve, VaultState},
    client::accounts,
    client::args,
};

// This only exists to solve the dependency mismatch between Anchor
// and solana_sdk. If Anchor updates to solana_sdk v3 then this can
// go away.
async fn get_v2_ata(wallet: &SolWallet, mint: &str) -> Result<Pubkey> {
    let v3 = wallet.get_ata(mint).await?;
    let v2 = Pubkey::from_str(&v3.to_string())?;

    Ok(v2)
}

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

    pub async fn get_vault_data_by_name(&self, name: &str) -> Result<KVaultInfo> {
        let vaults = self.api.get_all_kvaults().await?;

        let vault = vaults.into_iter().find(|v| v.state.name == name).unwrap();

        Ok(vault)
    }

    pub async fn get_vault_data_by_id(&self, vault_id: &str) -> Result<KVaultInfo> {
        let vaults = self.api.get_all_kvaults().await?;

        let vault = vaults.into_iter().find(|v| v.address == vault_id).unwrap();

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
            .expect("Couldn't convert keypair types");
        let user = Pubkey::from_str(&wallet.pubkey.to_string())?;

        let provider = Client::new_with_options(
            Cluster::Mainnet,
            Rc::new(&payer),
            CommitmentConfig::confirmed(),
        );

        let program = provider.program(kvault::ID)?;
        let state_addy = Pubkey::from_str_const(&vault_data.address);
        let vault_state: VaultState = program.account(state_addy).await?;

        let user_token_ata = get_v2_ata(&wallet, &vault_data.state.token_mint).await?;
        let user_shares_ata = get_v2_ata(&wallet, &vault_data.state.shares_mint).await?;

        let event_authority = get_event_authority(&program.id());

        let amnt = wallet
            .format_token(amount, &vault_data.state.token_mint)
            .await?;

        let mut remaining_accounts = Vec::new();

        let empty = Pubkey::default();
        for item in vault_state.vault_allocation_strategy.iter() {
            if item.reserve == empty {
                continue;
            }
            remaining_accounts.push(AccountMeta {
                pubkey: item.reserve,
                is_signer: false,
                is_writable: true,
            });
        }

        for item in vault_state.vault_allocation_strategy.iter() {
            if item.reserve == empty {
                continue;
            }
            let acct: Reserve = program.account(item.reserve).await?;
            remaining_accounts.push(AccountMeta {
                pubkey: acct.lending_market,
                is_signer: false,
                is_writable: false,
            });
        }

        let supply_ix = program
            .request()
            .accounts(accounts::Deposit {
                user,
                vault_state: state_addy,
                token_vault: vault_state.token_vault,
                token_mint: vault_state.token_mint,
                base_vault_authority: vault_state.base_vault_authority,
                shares_mint: vault_state.shares_mint,
                user_token_ata,
                user_shares_ata,
                klend_program: KLEND_ID,
                token_program: vault_state.token_program,
                shares_token_program: TOKEN_ID,
                event_authority,
                program: program.id(),
            })
            .accounts(remaining_accounts)
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
            Cluster::Mainnet,
            Rc::new(payer),
            CommitmentConfig::confirmed(),
        );

        let program = provider.program(kvault::ID)?;

        let token_mint = Pubkey::from_str_const(&vault_data.state.token_mint);
        let token_vault = Pubkey::from_str_const(&vault_data.state.token_vault);
        let base_vault_authority = Pubkey::from_str_const(&vault_data.state.base_vault_authority);
        let shares_mint = Pubkey::from_str_const(&vault_data.state.shares_mint);
        let token_program = Pubkey::from_str_const(&vault_data.state.token_program);
        let state_addy = Pubkey::from_str_const(&vault_data.address);
        let vault_state: VaultState = program.account(state_addy).await?;

        let user_token_ata = get_v2_ata(&wallet, &vault_data.state.token_mint).await?;
        let user_shares_ata = get_v2_ata(&wallet, &vault_data.state.shares_mint).await?;

        let event_authority = get_event_authority(&program.id());

        let amnt = wallet
            .format_token(amount, &vault_data.state.token_mint)
            .await?;

        let global_config =
            Pubkey::find_program_address(&["global_config".as_bytes()], &program.id()).0;

        let withdraw_from_available = accounts::WithdrawFromAvailable {
            user,
            vault_state: state_addy,
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

        let mut remaining_accounts = Vec::new();

        let empty = Pubkey::default();
        for item in vault_state.vault_allocation_strategy.iter() {
            if item.reserve == empty {
                continue;
            }
            remaining_accounts.push(AccountMeta {
                pubkey: item.reserve,
                is_signer: false,
                is_writable: true,
            });
        }

        for item in vault_state.vault_allocation_strategy.iter() {
            if item.reserve == empty {
                continue;
            }
            let acct: Reserve = program.account(item.reserve).await?;
            remaining_accounts.push(AccountMeta {
                pubkey: acct.lending_market,
                is_signer: false,
                is_writable: false,
            });
        }

        let reserve_addy = remaining_accounts.get(0).unwrap().pubkey;
        let ctoken = vault_data
            .state
            .vault_allocation_strategy
            .get(0)
            .unwrap()
            .ctoken_vault
            .clone();
        let reserve: Reserve = program.account(reserve_addy).await?;

        let withdraw_from_reserve_accounts = accounts::WithdrawFromReserveAccounts {
            vault_state: state_addy,
            reserve: reserve_addy,
            ctoken_vault: Pubkey::from_str(&ctoken)?,
            lending_market: reserve.lending_market,
            lending_market_authority: vault_state.vault_admin_authority,
            reserve_liquidity_supply: reserve.liquidity.supply_vault,
            reserve_collateral_mint: reserve.collateral.mint_pubkey,
            reserve_collateral_token_program: TOKEN_ID,
            instruction_sysvar_account: SYSVAR,
        };

        let supply_ix = program
            .request()
            .accounts(accounts::Withdraw {
                withdraw_from_available,
                withdraw_from_reserve_accounts,
                event_authority,
                program: program.id(),
            })
            .accounts(remaining_accounts)
            .args(args::Withdraw {
                shares_amount: amnt,
            })
            .instructions()?
            .remove(0);

        let _ = program.request().instruction(supply_ix).send().await?;

        Ok(())
    }
}
