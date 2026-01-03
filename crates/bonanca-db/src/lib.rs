pub mod transactions;
mod utils;

use std::path::{Path, PathBuf};

use anyhow::Result;
use bincode::{Decode, Encode};
use redb::{Database, ReadableDatabase, TableDefinition};

use crate::transactions::Txn;
use crate::utils::Bincode;

pub fn create_db(db_file: &Path) -> Result<()> {
    if !db_file.is_file() {
        let _ = Database::create(&db_file)?;
    }

    Ok(())
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct ChainInfo {
    pub name: String,
    pub rpc_url: String,
    pub wrapped_native: String,
    pub chain_id: Option<u16>,
}

pub struct BonancaDB {
    pub filename: PathBuf,
}

impl BonancaDB {
    pub fn new(filename: PathBuf) -> Self {
        Self { filename }
    }

    fn open(&self) -> Database {
        Database::open(&self.filename).unwrap()
    }

    pub fn get_api_key(&self, name: &str) -> Result<String> {
        let db = self.open();
        let api_table: TableDefinition<&str, &str> = TableDefinition::new("API Keys");

        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(api_table)?;

        let key = table.get(name)?.unwrap().value().to_string();

        Ok(key)
    }

    pub fn add_api_key(&self, name: &str, key: &str) -> Result<()> {
        let db = self.open();
        let write_txn = db.begin_write()?;
        let api_table: TableDefinition<&str, &str> = TableDefinition::new("API Keys");

        {
            let mut table = write_txn.open_table(api_table)?;

            table.insert(name, key)?;
        }

        write_txn.commit()?;

        Ok(())
    }

    pub fn write_chain_info(&self, chain: &str, info: ChainInfo) -> Result<()> {
        let db = self.open();
        let write_txn = db.begin_write()?;
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
        let db = self.open();
        let chain_info_table: TableDefinition<&str, Bincode<ChainInfo>> =
            TableDefinition::new("Chain Info");

        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(chain_info_table)?;

        let info = table.get(chain)?.unwrap().value();

        Ok(info)
    }

    pub fn write_txn(&self, chain: &str, child: u32, hash: &str, txn: Txn) -> Result<()> {
        let db = self.open();
        let write_txn = db.begin_write()?;
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
        let db = self.open();
        let write_txn = db.begin_write()?;
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
        let db = self.open();
        let table_name = format!("{}_{}", chain, child);
        let chain_table: TableDefinition<&str, Bincode<Txn>> = TableDefinition::new(&table_name);

        let read_txn = db.begin_read()?;
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
