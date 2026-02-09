mod args;
mod cmds;
mod db;
mod settings;

use clap::Parser;

use crate::{
    args::{GridBot, GridBotArgs},
    cmds::{balance, run},
};

#[tokio::main]
async fn main() {
    let args = GridBot::parse();

    match args.command {
        GridBotArgs::Balance(cmds) => balance(cmds).await.unwrap(),
        GridBotArgs::Run(cmds) => run(cmds).await.unwrap(),
    }
}
