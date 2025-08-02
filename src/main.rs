use clap::Parser;
use pendector::cli::Args;
use pendector::core::RepoScanner;
use pendector::output::OutputFormatter;

fn main() {
    let args = Args::parse();

    let scanner = RepoScanner::new();
    match scanner.scan_with_depth(&args.path, args.max_depth) {
        Ok(repositories) => {
            let filtered_repos: Vec<_> = if args.changes_only {
                repositories.into_iter().filter(|r| r.has_changes).collect()
            } else {
                repositories
            };

            let formatter = OutputFormatter::new(args.verbose);
            println!("{}", formatter.format_repositories(&filtered_repos));
        }
        Err(e) => {
            eprintln!("Error scanning repositories: {e}");
            std::process::exit(1);
        }
    }
}
