use std::path::PathBuf;

use anyhow::Result;
use redb::{Database, ReadableDatabase, TableDefinition};

pub struct ApiDB {
    pub filename: PathBuf,
}

impl ApiDB {
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
}
