use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Base directories to scan for repositories
    #[arg(default_values = &["."])]
    pub paths: Vec<String>,

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
}
