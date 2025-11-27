use anyhow::Result;
use bonanca_managers::loan_portfolio::LoanPortfolio;

use crate::lend::args::ShowArgs;

use super::args::{LendCommand, LendSubcommands};

pub async fn handle_lend_cmd(cmd: LendCommand) {
    match cmd.command {
        LendSubcommands::Show(cmd) => show_pools(cmd).await,
    }
}

async fn show_pools(cmd: ShowArgs) {
    let loan_port = LoanPortfolio::load(&cmd.json_file);

    let _ = loan_port.get_user_data().await;
}
