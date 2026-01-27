mod cli;
mod fetch_html;
mod grep;

pub use cli::UtilsCommand;

use anyhow::Result;

/// Run a utils subcommand
pub async fn run_command(cmd: UtilsCommand) -> Result<()> {
    match cmd {
        UtilsCommand::FetchHtml(args) => fetch_html::run(args).await,
        UtilsCommand::Grep(args) => grep::run(args),
    }
}
