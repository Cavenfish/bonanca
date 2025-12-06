use bonanca_core::config::{ApiKey, ChainInfo, Config};

use super::args::{ApiKeyArgs, ChainInfoArgs, ConfigCommand, ConfigSubcommands, KeyVaultArgs};

pub async fn handle_config_cmd(cmd: ConfigCommand) {
    match cmd.command {
        ConfigSubcommands::UpdateKeyvault(cmd) => update_keyvault(cmd),
        ConfigSubcommands::AddChainInfo(cmd) => add_chain_info(cmd),
        ConfigSubcommands::UpdateChainInfo(cmd) => update_chain_info(cmd),
        ConfigSubcommands::AddApiKey(cmd) => add_api_key(cmd),
        ConfigSubcommands::UpdateApiKey(cmd) => update_api_key(cmd),
    }
}

fn update_keyvault(cmd: KeyVaultArgs) {
    let config = Config::load();
    config.update_keyvault(cmd.filename);
}

fn add_chain_info(cmd: ChainInfoArgs) {
    let config = Config::load();
    let chain_info = ChainInfo {
        name: cmd.name,
        rpc_url: cmd.rpc_url,
        wrapped_native: cmd.wrapped_native,
        chain_id: cmd.chain_id,
    };

    config.add_chain_info(chain_info);
}

fn update_chain_info(cmd: ChainInfoArgs) {
    let config = Config::load();
    let chain_info = ChainInfo {
        name: cmd.name,
        rpc_url: cmd.rpc_url,
        wrapped_native: cmd.wrapped_native,
        chain_id: cmd.chain_id,
    };

    config.update_chain_info(chain_info);
}

fn add_api_key(cmd: ApiKeyArgs) {
    let config = Config::load();
    let api_key = ApiKey {
        name: cmd.name,
        key: cmd.key,
    };

    config.add_api_key(api_key);
}

fn update_api_key(cmd: ApiKeyArgs) {
    let config = Config::load();
    let api_key = ApiKey {
        name: cmd.name,
        key: cmd.key,
    };

    config.update_api_key(api_key);
}
