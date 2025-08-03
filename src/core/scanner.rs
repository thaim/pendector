use crate::core::Repository;
use crate::error::PendectorResult;
use crate::git::GitStatus;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

pub struct RepoScanner;

impl RepoScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RepoScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl RepoScanner {
    /// 指定のパス以下でリポジトリを再帰的に探索する
    pub fn scan<P: AsRef<Path>>(&self, base_path: P) -> PendectorResult<Vec<Repository>> {
        self.scan_with_depth(base_path, 10)
    }

    /// 指定の深さまでリポジトリを再帰的に探索する
    pub fn scan_with_depth<P: AsRef<Path>>(
        &self,
        base_path: P,
        max_depth: usize,
    ) -> PendectorResult<Vec<Repository>> {
        self.scan_with_options(base_path, max_depth, false)
    }

    /// 指定の深さとfetchオプションでリポジトリを再帰的に探索する
    pub fn scan_with_options<P: AsRef<Path>>(
        &self,
        base_path: P,
        max_depth: usize,
        should_fetch: bool,
    ) -> PendectorResult<Vec<Repository>> {
        let base_path = base_path.as_ref();
        let _base_path_str = base_path.to_string_lossy().to_string();

        // まずすべてのリポジトリパスを収集
        let repo_paths: Vec<PathBuf> = WalkDir::new(base_path)
            .follow_links(false)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| match e {
                Ok(entry) => Some(entry),
                Err(err) => {
                    eprintln!("Warning: Failed to access path during scan: {err}");
                    None
                }
            })
            .filter(|entry| entry.file_type().is_dir() && entry.file_name() == ".git")
            .filter_map(|entry| entry.path().parent().map(|p| p.to_path_buf()))
            .collect();

        // fetchが必要な場合は並列実行（プログレスバー付き）
        if should_fetch && !repo_paths.is_empty() {
            let _fetch_results = GitStatus::perform_parallel_fetch_with_progress(&repo_paths, true);
            // fetch結果は警告として出力されるので、ここでは特に処理しない
        }

        // 各リポジトリの状態を並列取得
        let repositories: Vec<Repository> = repo_paths
            .par_iter()
            .filter_map(|repo_path| {
                let mut repository = Repository::new(repo_path.clone());

                // Get git status information (fetchなしで実行)
                if let Ok(status) = GitStatus::get_repository_status(repo_path) {
                    repository = repository
                        .with_git_info(
                            status.has_changes,
                            status.current_branch,
                            status.changed_files,
                        )
                        .with_remote_info(
                            status.needs_pull,
                            status.needs_push,
                            status.remote_branch,
                        );
                }

                Some(repository)
            })
            .collect();

        Ok(repositories)
    }

    /// タイムアウト設定付きで指定の深さとfetchオプションでリポジトリを再帰的に探索する
    pub fn scan_with_options_and_timeout<P: AsRef<Path>>(
        &self,
        base_path: P,
        max_depth: usize,
        should_fetch: bool,
        fetch_timeout_secs: u64,
    ) -> PendectorResult<Vec<Repository>> {
        let base_path = base_path.as_ref();
        let _base_path_str = base_path.to_string_lossy().to_string();

        // まずすべてのリポジトリパスを収集
        let repo_paths: Vec<PathBuf> = WalkDir::new(base_path)
            .follow_links(false)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| match e {
                Ok(entry) => Some(entry),
                Err(err) => {
                    eprintln!("Warning: Failed to access path during scan: {err}");
                    None
                }
            })
            .filter(|entry| entry.file_type().is_dir() && entry.file_name() == ".git")
            .filter_map(|entry| entry.path().parent().map(|p| p.to_path_buf()))
            .collect();

        // fetchが必要な場合は並列実行（プログレスバー付き）
        if should_fetch && !repo_paths.is_empty() {
            let timeout = Duration::from_secs(fetch_timeout_secs);
            let _fetch_results = GitStatus::perform_parallel_fetch_with_timeout_and_progress(
                &repo_paths,
                timeout,
                true,
            );
            // fetch結果は警告として出力されるので、ここでは特に処理しない
        }

        // 各リポジトリの状態を並列取得
        let repositories: Vec<Repository> = repo_paths
            .par_iter()
            .filter_map(|repo_path| {
                let mut repository = Repository::new(repo_path.clone());

                // Get git status information (fetchなしで実行)
                if let Ok(status) = GitStatus::get_repository_status(repo_path) {
                    repository = repository
                        .with_git_info(
                            status.has_changes,
                            status.current_branch,
                            status.changed_files,
                        )
                        .with_remote_info(
                            status.needs_pull,
                            status.needs_push,
                            status.remote_branch,
                        );
                }

                Some(repository)
            })
            .collect();

        Ok(repositories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_finds_git_repositories() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create a mock git repository
        let repo_path = base_path.join("test_repo");
        fs::create_dir_all(&repo_path).unwrap();
        fs::create_dir_all(repo_path.join(".git")).unwrap();

        let scanner = RepoScanner::new();
        let repositories = scanner.scan(base_path).unwrap();

        assert_eq!(repositories.len(), 1);
        assert_eq!(repositories[0].name, "test_repo");
    }

    #[test]
    fn test_scan_ignores_non_git_directories() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create a regular directory
        let regular_dir = base_path.join("regular_dir");
        fs::create_dir_all(&regular_dir).unwrap();

        let scanner = RepoScanner::new();
        let repositories = scanner.scan(base_path).unwrap();

        assert_eq!(repositories.len(), 0);
    }

    #[test]
    fn test_scan_with_depth_limit() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create nested git repositories
        let shallow_repo = base_path.join("shallow_repo");
        fs::create_dir_all(&shallow_repo).unwrap();
        fs::create_dir_all(shallow_repo.join(".git")).unwrap();

        let deep_repo = base_path.join("level1").join("level2").join("deep_repo");
        fs::create_dir_all(&deep_repo).unwrap();
        fs::create_dir_all(deep_repo.join(".git")).unwrap();

        let scanner = RepoScanner::new();

        // Test with depth 0 - should find no repos (only base directory itself)
        let repositories = scanner.scan_with_depth(base_path, 0).unwrap();
        assert_eq!(repositories.len(), 0);

        // Test with depth 2 - should find shallow repo only
        let repositories = scanner.scan_with_depth(base_path, 2).unwrap();
        assert_eq!(repositories.len(), 1);
        assert_eq!(repositories[0].name, "shallow_repo");

        // Test with depth 4 - should find shallow and deep repos
        let repositories = scanner.scan_with_depth(base_path, 4).unwrap();
        assert_eq!(repositories.len(), 2);
    }
}
