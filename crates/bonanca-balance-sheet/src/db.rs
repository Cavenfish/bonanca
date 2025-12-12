use std::fmt::Debug;
use std::path::Path;
use std::{any::type_name, process::Child};

use anyhow::Result;
use bincode::{Decode, Encode, decode_from_slice, encode_to_vec};
use bonanca_core::transactions::Txn;
use redb::{
    Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition, Value,
};

pub fn create_db(db_file: &Path) -> Result<()> {
    if !db_file.is_file() {
        let _ = Database::create(&db_file)?;
    }

    Ok(())
}

pub fn write_txn(db_file: &Path, chain: &str, child: u32, hash: &str, txn: Txn) -> Result<()> {
    let db = Database::open(db_file)?;
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

pub fn write_txns(db_file: &Path, chain: &str, child: u32, txns: Vec<(String, Txn)>) -> Result<()> {
    let db = Database::open(db_file)?;
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

pub fn read_txns(db_file: &Path, chain: &str, child: u32) -> Result<Vec<(String, Txn)>> {
    let db = Database::open(db_file)?;
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

#[derive(Debug)]
pub struct Bincode<T>(pub T);

impl<T> Value for Bincode<T>
where
    T: Debug + Encode + Decode<()>,
{
    type SelfType<'a>
        = T
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        decode_from_slice(data, bincode::config::standard())
            .unwrap()
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        encode_to_vec(value, bincode::config::standard()).unwrap()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new(&format!("Bincode<{}>", type_name::<T>()))
    }
}
