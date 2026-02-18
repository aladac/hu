mod colors;
mod display;
mod service;
mod types;

use crate::shell::cli::LsArgs;
use anyhow::Result;

pub fn run(args: LsArgs) -> Result<()> {
    // We always inject -1 (one per line) when NOT in long mode and user did
    // not request a specific column format, so we get parseable output.
    let is_long = service::has_long_flag(&args.args);
    let is_single = service::has_single_column_flag(&args.args);

    let mut effective_args = args.args;

    // If not long and not already single-column, force -1 for parseable output
    if !is_long && !is_single {
        effective_args.insert(0, "-1".to_string());
    }

    // Run GNU ls with --color=never since we do our own coloring
    effective_args.insert(0, "--color=never".to_string());

    let stdout = service::execute_ls(&effective_args)?;
    let raw = String::from_utf8_lossy(&stdout);

    let enhanced = display::enhance_output(&raw, is_long);

    if !enhanced.is_empty() {
        println!("{}", enhanced);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_default_current_dir() {
        let args = LsArgs { args: vec![] };
        // May fail if gls not installed (macOS CI), that is acceptable
        let result = run(args);
        if service::detect_ls_binary() == "ls" {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_with_path() {
        let args = LsArgs {
            args: vec!["/tmp".to_string()],
        };
        let result = run(args);
        if service::detect_ls_binary() == "ls" {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_long_format() {
        let args = LsArgs {
            args: vec!["-l".to_string()],
        };
        let result = run(args);
        if service::detect_ls_binary() == "ls" {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_all_flag() {
        let args = LsArgs {
            args: vec!["-a".to_string()],
        };
        let result = run(args);
        if service::detect_ls_binary() == "ls" {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_combined_flags() {
        let args = LsArgs {
            args: vec!["-la".to_string()],
        };
        let result = run(args);
        if service::detect_ls_binary() == "ls" {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn run_nonexistent_path_fails() {
        let args = LsArgs {
            args: vec!["/nonexistent/xyz123".to_string()],
        };
        let result = run(args);
        assert!(result.is_err());
    }

    #[test]
    fn run_single_column_explicit() {
        let args = LsArgs {
            args: vec!["-1".to_string()],
        };
        let result = run(args);
        if service::detect_ls_binary() == "ls" {
            assert!(result.is_ok());
        }
    }
}
