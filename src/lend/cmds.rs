use bonanca_managers::loan_portfolio::LoanPortfolio;

use super::args::{BalArgs, LendCommand, LendSubcommands, ShowArgs};

pub async fn handle_lend_cmd(cmd: LendCommand) {
    match cmd.command {
        LendSubcommands::Show(cmd) => show_pools(cmd).await,
        LendSubcommands::Balance(cmd) => get_user_data(cmd).await,
    }
}

async fn show_pools(cmd: ShowArgs) {
    let loan_port = LoanPortfolio::load(&cmd.json_file);

    let _ = loan_port.get_token_pools().await;
}

async fn get_user_data(cmd: BalArgs) {
    let loan_port = LoanPortfolio::load(&cmd.json_file);

    let _ = loan_port.get_user_data().await;
}
