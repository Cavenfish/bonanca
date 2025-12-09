pub mod cashflows;
pub mod config;
pub mod holdings;
pub mod traits;

use dirs::data_dir;
use std::fs::create_dir_all;

use crate::config::Config;

pub fn init_config() {
    let config_dir = data_dir().unwrap().join("bonanca");

    if !config_dir.exists() {
        create_dir_all(&config_dir).unwrap();
    }

    let config_file = config_dir.join("config.json");

    if !config_file.exists() {
        let config = Config::default();

        config.write();
    }
}

pub fn get_default_config() -> Config {
    Config::load()
}
