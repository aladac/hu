mod around;
mod cli;
mod diff;
mod interface;
mod outline;
mod service;
mod types;

pub use cli::ReadArgs;

use anyhow::Result;

/// Run the read command
pub fn run(args: ReadArgs) -> Result<()> {
    service::run(args)
}
