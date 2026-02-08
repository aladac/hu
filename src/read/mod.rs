mod around;
mod cli;
mod diff;
mod display;
mod interface;
mod outline;
mod service;
mod types;

pub use cli::ReadArgs;
pub use types::ReadOutput;

use anyhow::Result;

/// Run the read command (CLI entry point - formats and prints)
#[cfg(not(tarpaulin_include))]
pub fn run(args: ReadArgs) -> Result<()> {
    let output = service::run(args)?;
    let formatted = display::format(&output);
    print!("{}", formatted);
    Ok(())
}

/// Run the read command and return data (for MCP/HTTP)
#[allow(dead_code)]
pub fn read(args: ReadArgs) -> Result<ReadOutput> {
    service::run(args)
}
