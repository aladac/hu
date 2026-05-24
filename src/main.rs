use clap::{CommandFactory, Parser};

mod cli;
mod context;
mod cron;
mod data;
mod docs;
mod git;
mod install;
mod mcp;
mod newrelic;
mod read;
mod setup;
mod shell;
mod util;
mod utils;

use cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => run_command(cmd).await,
        None => {
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
    }
}

async fn run_command(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::NewRelic { cmd: Some(cmd) } => {
            return newrelic::run(cmd).await;
        }
        Command::NewRelic { cmd: None } => {
            print_subcommand_help("newrelic")?;
        }
        Command::Utils { cmd: Some(cmd) } => {
            return utils::run_command(cmd).await;
        }
        Command::Utils { cmd: None } => {
            print_subcommand_help("utils")?;
        }
        Command::Context { cmd: Some(cmd) } => {
            return context::run_command(cmd).await;
        }
        Command::Context { cmd: None } => {
            print_subcommand_help("context")?;
        }
        Command::Read(args) => {
            return read::run(args);
        }
        Command::Data { cmd: Some(cmd) } => {
            return data::run_command(cmd).await;
        }
        Command::Data { cmd: None } => {
            print_subcommand_help("data")?;
        }
        Command::Install { cmd: Some(cmd) } => {
            return install::run_command(cmd).await;
        }
        Command::Install { cmd: None } => {
            print_subcommand_help("install")?;
        }
        Command::Docs { cmd: Some(cmd) } => {
            return docs::run_command(cmd).await;
        }
        Command::Docs { cmd: None } => {
            print_subcommand_help("docs")?;
        }
        Command::Cron { cmd: Some(cmd) } => {
            return cron::run_command(cmd);
        }
        Command::Cron { cmd: None } => {
            print_subcommand_help("cron")?;
        }
        Command::Shell { cmd: Some(cmd) } => {
            return shell::run_command(cmd);
        }
        Command::Shell { cmd: None } => {
            print_subcommand_help("shell")?;
        }
        Command::Mcp { cmd: Some(cmd) } => {
            return mcp::run_command(cmd).await;
        }
        Command::Mcp { cmd: None } => {
            print_subcommand_help("mcp")?;
        }
        Command::Setup { cmd: Some(cmd) } => {
            return setup::run_command(cmd).await;
        }
        Command::Setup { cmd: None } => {
            print_subcommand_help("setup")?;
        }
    }
    Ok(())
}

fn print_subcommand_help(name: &str) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    for sub in cmd.get_subcommands_mut() {
        if sub.get_name() == name {
            sub.print_help()?;
            println!();
            return Ok(());
        }
    }
    unreachable!("unknown subcommand: {}", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_no_args() {
        let cli = Cli::try_parse_from::<[&str; 0], &str>([]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_subcommand_without_action() {
        let cli = Cli::try_parse_from(["hu", "newrelic"]).unwrap();
        assert!(matches!(cli.command, Some(Command::NewRelic { cmd: None })));
    }

    #[test]
    fn parses_command_aliases() {
        // nr -> newrelic
        let cli = Cli::try_parse_from(["hu", "nr", "incidents"]).unwrap();
        assert!(matches!(cli.command, Some(Command::NewRelic { .. })));
    }
}
