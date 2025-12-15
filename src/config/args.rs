use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct ConfigCommand {
    #[clap(subcommand)]
    pub command: ConfigSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommands {
    /// Update default keyvault filename
    UpdateKeyvault(KeyVaultArgs),

    /// Add or update chain information
    AddChainInfo(ChainInfoArgs),

    /// Add or update api information
    AddApiKey(ApiKeyArgs),

    /// Show
    Show,
}

#[derive(Debug, Args)]
pub struct KeyVaultArgs {
    /// New keyvault filename
    pub filename: PathBuf,
}

#[derive(Debug, Args)]
pub struct ChainInfoArgs {
    /// Chain name
    pub name: String,

    /// Default chain rpc
    pub rpc_url: String,

    /// Wrapped native token address
    pub wrapped_native: String,

    /// Chain id (EVM only)
    pub chain_id: Option<u16>,
}

#[derive(Debug, Args)]
pub struct ApiKeyArgs {
    /// API name
    pub name: String,

    /// API key
    pub key: String,
}
