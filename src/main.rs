use clap::Parser;
use pendector::cli::Args;
use pendector::config::Config;
use pendector::core::RepoScanner;
use pendector::output::OutputFormatter;
use pendector::PendectorError;
use std::path::Path;

fn main() {
    let args = Args::parse();

    // 設定ファイルの読み込み
    let config = if args.no_config {
        Config::default()
    } else {
        let config_path = args.config.as_ref().map(Path::new);
        match Config::load(config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: {e}");
                eprintln!("Using default configuration");
                Config::default()
            }
        }
    };

    let scanner = RepoScanner::new();
    let mut all_repositories = Vec::new();

    // パスの決定：CLI引数とフラグに基づく
    let paths_to_scan = if args.paths.is_empty() {
        // パスが指定されていない場合は設定ファイルのデフォルトパスを使用
        config.get_default_paths().to_vec()
    } else if args.add_path {
        // --add-pathフラグがある場合は設定ファイルのパスに追加
        let mut combined_paths = config.get_default_paths().to_vec();
        combined_paths.extend(args.paths.iter().cloned());
        combined_paths
    } else {
        // 通常は指定されたパスのみを使用（設定ファイルのパスは上書き）
        args.paths.clone()
    };

    for path in &paths_to_scan {
        // チルダ展開してからパスの存在確認
        let expanded_path = shellexpand::tilde(path);
        let path_buf = std::path::Path::new(expanded_path.as_ref());
        if !path_buf.exists() {
            eprintln!("Error: Path '{path}' does not exist");
            std::process::exit(1);
        }
        if !path_buf.is_dir() {
            eprintln!("Error: Path '{path}' is not a directory");
            std::process::exit(1);
        }

        // パス固有の設定を取得（設定ファイルのパスでない場合はデフォルト設定のみ）
        let path_config = if args.paths.is_empty() || args.add_path {
            // 設定ファイルのパスを使用している場合はパス固有設定を適用
            config.get_path_config(path)
        } else {
            // CLI引数で上書きした場合はデフォルト設定のみ使用
            use pendector::config::PathConfigResolved;
            PathConfigResolved {
                max_depth: config.defaults.max_depth,
                fetch: config.defaults.fetch,
                fetch_timeout: config.defaults.fetch_timeout,
                format: config.defaults.format.clone(),
                verbose: config.defaults.verbose,
                changes_only: config.defaults.changes_only,
            }
        };

        // CLI引数が設定ファイルより優先
        let max_depth = if args.max_depth != 3 {
            args.max_depth
        } else {
            path_config.max_depth
        };

        let fetch = if args.fetch { true } else { path_config.fetch };

        let fetch_timeout = if args.fetch_timeout != 5 {
            args.fetch_timeout
        } else {
            path_config.fetch_timeout
        };

        match scanner.scan_with_options_and_timeout(
            expanded_path.as_ref(),
            max_depth,
            fetch,
            fetch_timeout,
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

    // フィルタリングの決定：CLI引数 > パス固有設定 > デフォルト設定
    let changes_only = if args.changes_only {
        true
    } else {
        // パス固有設定は複数パスがある場合複雑になるので、最初のパスの設定を使用
        paths_to_scan
            .first()
            .map(|path| config.get_path_config(path).changes_only)
            .unwrap_or(false)
    };

    let filtered_repos: Vec<_> = if changes_only {
        all_repositories
            .into_iter()
            .filter(|r| r.has_changes)
            .collect()
    } else {
        all_repositories
    };

    // 出力フォーマットの決定：CLI引数 > パス固有設定 > デフォルト設定
    let format = if args.format != "text" {
        args.format.clone()
    } else {
        paths_to_scan
            .first()
            .map(|path| config.get_path_config(path).format)
            .unwrap_or_else(|| "text".to_string())
    };

    // verboseモードの決定：CLI引数 > パス固有設定 > デフォルト設定
    let verbose = if args.verbose {
        true
    } else {
        paths_to_scan
            .first()
            .map(|path| config.get_path_config(path).verbose)
            .unwrap_or(false)
    };

    let formatter = OutputFormatter::new(verbose, format);
    println!("{}", formatter.format_repositories(&filtered_repos));
}
