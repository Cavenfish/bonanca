pub mod db;

use bonanca_core::config::Config;

use crate::db::create_db;

pub fn init_database() {
    let config = Config::load();

    create_db(&config.database).unwrap();
}
