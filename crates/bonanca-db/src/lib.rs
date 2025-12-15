mod utils;

use std::path::Path;

use anyhow::Result;
use bincode::{Decode, Encode};
use bonanca_core::{config::Config, transactions::Txn};
use redb::{Database, ReadableDatabase, TableDefinition};

use crate::utils::{Bincode, create_db};

pub fn init_database() {
    let config = Config::load();

    create_db(&config.database).unwrap();
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct ChainInfo {
    pub name: String,
    pub rpc_url: String,
    pub wrapped_native: String,
    pub chain_id: Option<u16>,
}

pub struct BonancaDB {
    pub db: Database,
}

impl BonancaDB {
    pub fn load() -> Self {
        let config = Config::load();
        let db = Database::open(config.database).unwrap();

        Self { db }
    }

    pub fn new(filename: &Path) -> Self {
        let db = Database::open(filename).expect("Couldn't open database file");

        Self { db }
    }

    pub fn get_api_key(&self, name: &str) -> Result<String> {
        let api_table: TableDefinition<&str, &str> = TableDefinition::new("API Keys");

        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(api_table)?;

        let key = table.get(name)?.unwrap().value().to_string();

        Ok(key)
    }

    pub fn add_api_key(&self, name: &str, key: &str) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        let api_table: TableDefinition<&str, &str> = TableDefinition::new("API Keys");

        {
            let mut table = write_txn.open_table(api_table)?;

            table.insert(name, key)?;
        }

        write_txn.commit()?;

        Ok(())
    }

    pub fn write_chain_info(&self, chain: &str, info: ChainInfo) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        let chain_info_table: TableDefinition<&str, Bincode<ChainInfo>> =
            TableDefinition::new("Chain Info");

        {
            let mut table = write_txn.open_table(chain_info_table)?;

            table.insert(chain, info)?;
        }

        write_txn.commit()?;

        Ok(())
    }

    pub fn read_chain_info(&self, chain: &str) -> Result<ChainInfo> {
        let chain_info_table: TableDefinition<&str, Bincode<ChainInfo>> =
            TableDefinition::new("Chain Info");

        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(chain_info_table)?;

        let info = table.get(chain)?.unwrap().value();

        Ok(info)
    }

    pub fn write_txn(&self, chain: &str, child: u32, hash: &str, txn: Txn) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        let table_name = format!("{}_{}", chain, child);
        let chain_table: TableDefinition<&str, Bincode<Txn>> = TableDefinition::new(&table_name);

        {
            let mut table = write_txn.open_table(chain_table)?;

            table.insert(hash, &txn)?;
        }

        write_txn.commit()?;

        Ok(())
    }

    pub fn write_txns(&self, chain: &str, child: u32, txns: Vec<(String, Txn)>) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        let table_name = format!("{}_{}", chain, child);
        let chain_table: TableDefinition<&str, Bincode<Txn>> = TableDefinition::new(&table_name);

        {
            let mut table = write_txn.open_table(chain_table)?;

            let _ = txns.into_iter().for_each(|(hash, txn)| {
                table.insert(hash.as_str(), &txn).unwrap();
            });
        }

        write_txn.commit()?;

        Ok(())
    }

    pub fn read_txns(&self, chain: &str, child: u32) -> Result<Vec<(String, Txn)>> {
        let table_name = format!("{}_{}", chain, child);
        let chain_table: TableDefinition<&str, Bincode<Txn>> = TableDefinition::new(&table_name);

        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(chain_table)?;

        let tmp: Vec<(String, Txn)> = table
            .range::<&str>(..)?
            .map(|r| {
                (
                    r.as_ref().unwrap().0.value().to_string(),
                    r.unwrap().1.value(),
                )
            })
            .collect();

        Ok(tmp)
    }
}
