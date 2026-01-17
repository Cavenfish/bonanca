use std::path::PathBuf;

use anyhow::Result;
use bincode::{Decode, Encode};
use redb::{Database, ReadableDatabase, TableDefinition};

use crate::utils::Bincode;

#[derive(Clone, Debug, Encode, Decode)]
pub struct ChainInfo {
    pub name: String,
    pub rpc_url: String,
    pub wrapped_native: String,
    pub chain_id: Option<u16>,
}

pub struct ChainsDB {
    pub filename: PathBuf,
}

impl ChainsDB {
    pub fn new(filename: PathBuf) -> Self {
        Self { filename }
    }

    pub fn create_db(&self) -> Result<()> {
        if !self.filename.is_file() {
            let _ = Database::create(&self.filename)?;
        }

        Ok(())
    }

    fn open(&self) -> Database {
        Database::open(&self.filename).unwrap()
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
}
