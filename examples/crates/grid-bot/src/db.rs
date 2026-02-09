use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use anyhow::Result;
use bonanca::defi::CoW;
use serde::{Deserialize, Serialize};

use crate::settings::GridBotSettings;

#[derive(Debug, Deserialize, Serialize)]
pub struct GridBotLog {
    pub active_orders: Vec<String>,
    pub trades: HashMap<String, i64>,
}

impl GridBotLog {
    pub fn init(settings: &GridBotSettings) -> Self {
        let mut trades = HashMap::new();
        trades.insert(settings.trading_pair.token_a.address.clone(), 0);
        trades.insert(settings.trading_pair.token_b.address.clone(), 0);

        Self {
            active_orders: vec![],
            trades,
        }
    }

    pub fn load(settings: &GridBotSettings) -> Self {
        if settings.log_file.exists() {
            let file = File::open(&settings.log_file).expect("Could not open file");
            let reader = BufReader::new(file);

            serde_json::from_reader(reader).expect("Check JSON file")
        } else {
            Self::init(settings)
        }
    }

    pub fn write(&self, fname: &Path) -> Result<()> {
        let file = File::create(fname)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
    }

    fn update_amount(&mut self, token: String, amount: i64) -> Result<()> {
        let old = self.trades.get_mut(&token).expect("Token log not found");
        *old += amount;
        Ok(())
    }

    pub async fn prune_log(&mut self, chain: &str) -> Result<()> {
        let cow = CoW::new(chain)?;

        let mut i = 0;
        while i < self.active_orders.len() {
            let uid = &self.active_orders[i];
            let info = cow.get_order_info(&uid).await?;

            if info.status == "fulfilled" {
                self.active_orders.remove(i);

                let sell_amount: i64 = info.sell_amount.parse()?;

                self.update_amount(info.sell_token, sell_amount * -1)?;
                self.update_amount(info.buy_token, info.buy_amount.parse()?)?;
            } else {
                i += 1;
            }
        }

        Ok(())
    }
}
