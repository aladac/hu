use clap::Args;

#[derive(Debug, Args)]
pub struct ReadArgs {
    /// File path to read
    pub path: String,

    /// Show file outline (functions, structs, classes)
    #[arg(long, short = 'o')]
    pub outline: bool,

    /// Show public interface only (pub items in Rust, exports in JS)
    #[arg(long, short = 'i')]
    pub interface: bool,

    /// Show lines around a specific line number
    #[arg(long, short = 'a', value_name = "LINE")]
    pub around: Option<usize>,

    /// Number of context lines for --around (default: 10)
    #[arg(long, short = 'n', default_value = "10")]
    pub context: usize,

    /// Show git diff
    #[arg(long, short = 'd')]
    pub diff: bool,

    /// Commit to diff against (default: HEAD)
    #[arg(long, default_value = "HEAD")]
    pub commit: String,
}

impl ReadArgs {
    /// Check if any mode is active (outline, interface, around, diff)
    #[cfg(test)]
    pub fn has_mode(&self) -> bool {
        self.outline || self.interface || self.around.is_some() || self.diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        read: ReadArgs,
    }

    #[test]
    fn parse_path_only() {
        let cli = TestCli::try_parse_from(["test", "file.rs"]).unwrap();
        assert_eq!(cli.read.path, "file.rs");
        assert!(!cli.read.outline);
        assert!(!cli.read.interface);
        assert!(cli.read.around.is_none());
        assert!(!cli.read.diff);
    }

    #[test]
    fn parse_outline_long() {
        let cli = TestCli::try_parse_from(["test", "--outline", "file.rs"]).unwrap();
        assert!(cli.read.outline);
    }

    #[test]
    fn parse_outline_short() {
        let cli = TestCli::try_parse_from(["test", "-o", "file.rs"]).unwrap();
        assert!(cli.read.outline);
    }

    #[test]
    fn parse_interface_long() {
        let cli = TestCli::try_parse_from(["test", "--interface", "file.rs"]).unwrap();
        assert!(cli.read.interface);
    }

    #[test]
    fn parse_interface_short() {
        let cli = TestCli::try_parse_from(["test", "-i", "file.rs"]).unwrap();
        assert!(cli.read.interface);
    }

    #[test]
    fn parse_around_long() {
        let cli = TestCli::try_parse_from(["test", "--around", "50", "file.rs"]).unwrap();
        assert_eq!(cli.read.around, Some(50));
    }

    #[test]
    fn parse_around_short() {
        let cli = TestCli::try_parse_from(["test", "-a", "100", "file.rs"]).unwrap();
        assert_eq!(cli.read.around, Some(100));
    }

    #[test]
    fn parse_context_long() {
        let cli = TestCli::try_parse_from(["test", "-a", "50", "-n", "20", "file.rs"]).unwrap();
        assert_eq!(cli.read.context, 20);
    }

    #[test]
    fn parse_context_default() {
        let cli = TestCli::try_parse_from(["test", "file.rs"]).unwrap();
        assert_eq!(cli.read.context, 10);
    }

    #[test]
    fn parse_diff_flag() {
        let cli = TestCli::try_parse_from(["test", "--diff", "file.rs"]).unwrap();
        assert!(cli.read.diff);
        assert_eq!(cli.read.commit, "HEAD");
    }

    #[test]
    fn parse_diff_with_commit() {
        let cli =
            TestCli::try_parse_from(["test", "--diff", "--commit", "HEAD~1", "file.rs"]).unwrap();
        assert!(cli.read.diff);
        assert_eq!(cli.read.commit, "HEAD~1");
    }

    #[test]
    fn has_mode_none() {
        let cli = TestCli::try_parse_from(["test", "file.rs"]).unwrap();
        assert!(!cli.read.has_mode());
    }

    #[test]
    fn has_mode_outline() {
        let cli = TestCli::try_parse_from(["test", "-o", "file.rs"]).unwrap();
        assert!(cli.read.has_mode());
    }

    #[test]
    fn has_mode_interface() {
        let cli = TestCli::try_parse_from(["test", "-i", "file.rs"]).unwrap();
        assert!(cli.read.has_mode());
    }

    #[test]
    fn has_mode_around() {
        let cli = TestCli::try_parse_from(["test", "-a", "50", "file.rs"]).unwrap();
        assert!(cli.read.has_mode());
    }

    #[test]
    fn has_mode_diff() {
        let cli = TestCli::try_parse_from(["test", "-d", "file.rs"]).unwrap();
        assert!(cli.read.has_mode());
    }

    #[test]
    fn read_args_debug() {
        let cli = TestCli::try_parse_from(["test", "file.rs"]).unwrap();
        let debug = format!("{:?}", cli.read);
        assert!(debug.contains("ReadArgs"));
    }

    #[test]
    fn path_required() {
        let result = TestCli::try_parse_from(["test"]);
        assert!(result.is_err());
    }

    #[test]
    fn combined_flags() {
        let cli = TestCli::try_parse_from(["test", "-o", "-i", "-a", "50", "-n", "5", "file.rs"])
            .unwrap();
        assert!(cli.read.outline);
        assert!(cli.read.interface);
        assert_eq!(cli.read.around, Some(50));
        assert_eq!(cli.read.context, 5);
    }
}
