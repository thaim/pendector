use clap::Parser;
use pendector::cli::Args;
use pendector::core::RepoScanner;
use pendector::output::OutputFormatter;

fn main() {
    let args = Args::parse();

    let scanner = RepoScanner::new();
    let mut all_repositories = Vec::new();

    for path in &args.paths {
        match scanner.scan_with_options(path, args.max_depth, args.fetch) {
            Ok(mut repositories) => {
                all_repositories.append(&mut repositories);
            }
            Err(e) => {
                eprintln!("Error scanning path '{path}': {e}");
                std::process::exit(1);
            }
        }
    }

    let filtered_repos: Vec<_> = if args.changes_only {
        all_repositories
            .into_iter()
            .filter(|r| r.has_changes)
            .collect()
    } else {
        all_repositories
    };

    let formatter = OutputFormatter::new(args.verbose, args.format);
    println!("{}", formatter.format_repositories(&filtered_repos));
}
