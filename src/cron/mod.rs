mod cli;
mod display;
mod service;
mod types;

pub use cli::CronCommand;

use anyhow::Result;

use cli::{AddArgs, ListArgs, RemoveArgs};
use types::Schedule;

/// Run a cron subcommand
pub fn run_command(cmd: CronCommand) -> Result<()> {
    match cmd {
        CronCommand::Add(args) => run_add(args),
        CronCommand::List(args) => run_list(args),
        CronCommand::Remove(args) => run_remove(args),
    }
}

fn run_add(args: AddArgs) -> Result<()> {
    let schedule = Schedule::parse(&args.schedule).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid schedule '{}'. Use: hourly, daily, weekly, monthly, reboot",
            args.schedule
        )
    })?;

    let job = service::add_job(schedule, &args.command)?;
    println!("{}", display::format_added(&job, args.json));
    Ok(())
}

fn run_list(args: ListArgs) -> Result<()> {
    let jobs = service::list_jobs(args.hu_only)?;
    println!("{}", display::format_jobs(&jobs, args.json));
    Ok(())
}

fn run_remove(args: RemoveArgs) -> Result<()> {
    let jobs = service::list_jobs(false)?;
    let matching: Vec<_> = jobs.iter().filter(|j| j.matches(&args.pattern)).collect();

    if matching.is_empty() {
        println!("{}", display::format_removed(&[], args.json));
        return Ok(());
    }

    if !args.force && !args.json {
        println!("Will remove {} job(s):", matching.len());
        for job in &matching {
            println!("  - {} {}", job.expression, job.command);
        }
        println!("\nUse --force to confirm removal");
        return Ok(());
    }

    let removed = service::remove_jobs(&args.pattern)?;
    println!("{}", display::format_removed(&removed, args.json));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cron_command_exported() {
        let _ = std::any::type_name::<CronCommand>();
    }

    #[test]
    fn schedule_parse_in_add() {
        // Test that invalid schedule produces error message
        let result = Schedule::parse("invalid");
        assert!(result.is_none());
    }

    #[test]
    fn schedule_parse_valid() {
        assert!(Schedule::parse("daily").is_some());
        assert!(Schedule::parse("hourly").is_some());
        assert!(Schedule::parse("weekly").is_some());
        assert!(Schedule::parse("monthly").is_some());
        assert!(Schedule::parse("reboot").is_some());
    }
}
