use clap::Parser;
use pendector::cli::Args;
use pendector::core::RepoScanner;
use pendector::output::OutputFormatter;
use pendector::PendectorError;

fn main() {
    let args = Args::parse();

    let scanner = RepoScanner::new();
    let mut all_repositories = Vec::new();

    for path in &args.paths {
        // パスの存在確認
        let path_buf = std::path::Path::new(path);
        if !path_buf.exists() {
            eprintln!("Error: Path '{path}' does not exist");
            std::process::exit(1);
        }
        if !path_buf.is_dir() {
            eprintln!("Error: Path '{path}' is not a directory");
            std::process::exit(1);
        }

        match scanner.scan_with_options_and_timeout(
            path,
            args.max_depth,
            args.fetch,
            args.fetch_timeout,
        ) {
            Ok(mut repositories) => {
                all_repositories.append(&mut repositories);
            }
            Err(e) => {
                match &e {
                    PendectorError::GitRepositoryNotFound(_) => {
                        eprintln!("Warning: {e}");
                        // Git repository not found は続行
                    }
                    PendectorError::FileSystemError { .. } => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                    _ => {
                        eprintln!("Error scanning path '{path}': {e}");
                        std::process::exit(1);
                    }
                }
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
