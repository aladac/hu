use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum EksCommand {
    /// List pods
    List,
    /// Exec into a pod
    Exec {
        /// Pod number from list
        #[arg(short, long)]
        pod: Option<usize>,
    },
    /// Tail logs from pods
    Logs,
}
