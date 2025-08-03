use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Base directories to scan for repositories
    pub paths: Vec<String>,

    /// Add specified paths to config file default paths instead of replacing them
    #[arg(short = 'a', long = "add-path")]
    pub add_path: bool,

    /// Show only repositories with changes
    #[arg(short = 'c', long)]
    pub changes_only: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Maximum depth for recursive directory search
    #[arg(short = 'd', long, default_value = "3")]
    pub max_depth: usize,

    /// Output format: text or json
    #[arg(short = 'f', long, default_value = "text")]
    pub format: String,

    /// Fetch from remote before checking sync status
    #[arg(long)]
    pub fetch: bool,

    /// Timeout for fetch operations in seconds
    #[arg(long, default_value = "5")]
    pub fetch_timeout: u64,

    /// Path to configuration file
    #[arg(long)]
    pub config: Option<String>,

    /// Ignore configuration file
    #[arg(long)]
    pub no_config: bool,

    /// Exclude patterns (can be specified multiple times)
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Ignore exclude patterns from configuration file
    #[arg(long)]
    pub no_exclude: bool,
}
