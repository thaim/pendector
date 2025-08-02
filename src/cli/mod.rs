use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Base directory to scan for repositories
    #[arg(default_value = ".")]
    pub path: String,

    /// Show only repositories with changes
    #[arg(short = 'c', long)]
    pub changes_only: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Maximum depth for recursive directory search
    #[arg(short = 'd', long, default_value = "3")]
    pub max_depth: usize,
}
