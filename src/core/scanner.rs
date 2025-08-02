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
        let mut repositories = Vec::new();

        for entry in WalkDir::new(base_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() && entry.file_name() == ".git" {
                if let Some(repo_path) = entry.path().parent() {
                    let mut repository = Repository::new(repo_path.to_path_buf());

                    // Get git status information
                    if let Ok(status) = GitStatus::get_repository_status(repo_path) {
                        repository = repository.with_git_info(
                            status.has_changes,
                            status.current_branch,
                            status.changed_files,
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
}
