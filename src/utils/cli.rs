use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum UtilsCommand {
    /// Fetch URL and convert to markdown
    FetchHtml(FetchHtmlArgs),
    /// Smart grep with token-saving options
    Grep(GrepArgs),
    /// Web search using Brave Search API
    WebSearch(WebSearchArgs),
}

#[derive(Debug, Args)]
pub struct FetchHtmlArgs {
    /// URL to fetch
    pub url: String,

    /// Extract main content only (strip nav, footer, scripts, ads)
    #[arg(long, short = 'c')]
    pub content: bool,

    /// Return summary (first N paragraphs + headings)
    #[arg(long, short = 's')]
    pub summary: bool,

    /// Extract links only [text](url)
    #[arg(long, short = 'l')]
    pub links: bool,

    /// Extract headings only (document outline)
    #[arg(long, short = 'H')]
    pub headings: bool,

    /// CSS selector to target (e.g., "article", "main", ".content")
    #[arg(long)]
    pub selector: Option<String>,

    /// Output to file instead of stdout
    #[arg(long, short = 'o')]
    pub output: Option<String>,

    /// Raw output (no filtering)
    #[arg(long, short = 'r')]
    pub raw: bool,
}

#[derive(Debug, Args)]
pub struct GrepArgs {
    /// Pattern to search for (regex)
    pub pattern: String,

    /// Path to search (default: current directory)
    #[arg(default_value = ".")]
    pub path: String,

    /// Return file:line references only (no content)
    #[arg(long)]
    pub refs: bool,

    /// Deduplicate similar matches
    #[arg(long)]
    pub unique: bool,

    /// Sort by relevance (match density)
    #[arg(long)]
    pub ranked: bool,

    /// Limit number of results
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,

    /// Show function/class signature only (not body)
    #[arg(long)]
    pub signature: bool,

    /// File glob pattern (e.g., "*.rs", "*.py")
    #[arg(long, short = 'g')]
    pub glob: Option<String>,

    /// Case insensitive search
    #[arg(long, short = 'i')]
    pub ignore_case: bool,

    /// Include hidden files
    #[arg(long)]
    pub hidden: bool,
}

#[derive(Debug, Args)]
pub struct WebSearchArgs {
    /// Search query
    pub query: String,

    /// Number of results to fetch content from (default: 3)
    #[arg(long, short = 'n', default_value = "3")]
    pub results: usize,

    /// Only show search results (don't fetch content)
    #[arg(long, short = 'l')]
    pub list: bool,

    /// Output to file instead of stdout
    #[arg(long, short = 'o')]
    pub output: Option<String>,
}
