use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum NewRelicCommand {
    /// List open incidents
    Incidents,
    /// Run NRQL query
    Query {
        /// NRQL query string
        nrql: String,
    },
}
