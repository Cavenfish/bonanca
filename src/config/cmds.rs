use bonanca_core::config::Config;
use bonanca_db::{BonancaDB, ChainInfo};

use super::args::{ApiKeyArgs, ChainInfoArgs, ConfigCommand, ConfigSubcommands, KeyVaultArgs};

pub async fn handle_config_cmd(cmd: ConfigCommand) {
    match cmd.command {
        ConfigSubcommands::UpdateKeyvault(cmd) => update_keyvault(cmd),
        ConfigSubcommands::AddChainInfo(cmd) => add_chain_info(cmd),
        ConfigSubcommands::AddApiKey(cmd) => add_api_key(cmd),
        ConfigSubcommands::Show => print_config(),
    }
}

fn print_config() {
    let config = Config::load();
    println!("Keyvault: {}", config.keyvault.to_str().unwrap());

    // config
    //     .api_keys
    //     .iter()
    //     .for_each(|k| println!("{}: {}", k.name, k.key));
}

fn update_keyvault(cmd: KeyVaultArgs) {
    let config = Config::load();
    config.update_keyvault(cmd.filename);
}

fn add_chain_info(cmd: ChainInfoArgs) {
    let config = Config::load();
    let db = BonancaDB::new(&config.database);
    let chain = cmd.name.clone();

    let chain_info = ChainInfo {
        name: cmd.name,
        rpc_url: cmd.rpc_url,
        wrapped_native: cmd.wrapped_native,
        chain_id: cmd.chain_id,
    };

    db.write_chain_info(&chain, chain_info).unwrap();
}

fn add_api_key(cmd: ApiKeyArgs) {
    let config = Config::load();
    let db = BonancaDB::new(&config.database);

    db.add_api_key(&cmd.name, &cmd.key);
}
