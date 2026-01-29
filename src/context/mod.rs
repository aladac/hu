mod cli;
mod service;
mod store;
mod types;

pub use cli::ContextCommand;

use anyhow::Result;

/// Run a context subcommand
pub async fn run_command(cmd: ContextCommand) -> Result<()> {
    match cmd {
        ContextCommand::Track(args) => service::track(&args.paths).await,
        ContextCommand::Check(args) => service::check(&args.paths).await,
        ContextCommand::Summary => service::summary().await,
        ContextCommand::Clear => service::clear().await,
    }
}
