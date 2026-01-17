use std::path::PathBuf;

use anyhow::Result;
use bincode::{Decode, Encode};
use redb::{Database, ReadableDatabase, TableDefinition};

use crate::utils::Bincode;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Txn {
    pub pubkey: String,
    pub block: u64,
    pub timestamp: u64,
    pub gas_used: f64,
    pub operation: CryptoOperation,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum CryptoOperation {
    Approve(EvmApprove),
    Trade(CryptoTrade),
    Transfer(CryptoTransfer),
    Banking(CryptoBanking),
    Liquidity(CryptoLP),
    None,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct EvmApprove {
    pub token: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CryptoTrade {
    pub sell: String,
    pub buy: String,
    pub sell_amount: f64,
    pub buy_amount: f64,
    // pub sell_value: f64,
    // pub buy_value: f64,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CryptoTransfer {
    pub token: String,
    pub amount: f64,
    // pub value: f64,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CryptoBanking {
    pub token: String,
    pub amount: f64,
    pub action: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CryptoLP {
    pub token_a: String,
    pub token_b: String,
    pub amount_a: f64,
    pub amount_b: f64,
    // pub total_value: f64,
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
