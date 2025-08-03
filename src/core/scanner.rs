use crate::core::Repository;
use crate::git::GitStatus;
use std::path::Path;
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
    pub fn scan<P: AsRef<Path>>(
        &self,
        base_path: P,
    ) -> Result<Vec<Repository>, Box<dyn std::error::Error>> {
        self.scan_with_depth(base_path, 10)
    }

    /// 指定の深さまでリポジトリを再帰的に探索する
    pub fn scan_with_depth<P: AsRef<Path>>(
        &self,
        base_path: P,
        max_depth: usize,
    ) -> Result<Vec<Repository>, Box<dyn std::error::Error>> {
        self.scan_with_options(base_path, max_depth, false)
    }

    /// 指定の深さとfetchオプションでリポジトリを再帰的に探索する
    pub fn scan_with_options<P: AsRef<Path>>(
        &self,
        base_path: P,
        max_depth: usize,
        should_fetch: bool,
    ) -> Result<Vec<Repository>, Box<dyn std::error::Error>> {
        let mut repositories = Vec::new();

        for entry in WalkDir::new(base_path)
            .follow_links(false)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() && entry.file_name() == ".git" {
                if let Some(repo_path) = entry.path().parent() {
                    let mut repository = Repository::new(repo_path.to_path_buf());

                    // Get git status information
                    if let Ok(status) =
                        GitStatus::get_repository_status_with_fetch(repo_path, should_fetch)
                    {
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

                    repositories.push(repository);
                }
            }
        }

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
