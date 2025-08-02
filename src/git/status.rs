use git2::{Repository as Git2Repository, StatusOptions};
use std::path::Path;

pub struct GitStatus;

impl GitStatus {
    /// git2ライブラリを使用してリポジトリの状態を取得
    pub fn get_repository_status<P: AsRef<Path>>(
        repo_path: P,
    ) -> Result<(bool, Option<String>, Vec<String>), Box<dyn std::error::Error>> {
        let repo_path = repo_path.as_ref();

        // git2でリポジトリを開く
        let repo = Git2Repository::open(repo_path)?;

        // 現在のブランチ名を取得
        let current_branch = if let Ok(head) = repo.head() {
            if let Some(name) = head.shorthand() {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        };

        // ステータス情報を取得
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .include_ignored(false)
            .renames_head_to_index(false)
            .renames_index_to_workdir(false);

        let statuses = repo.statuses(Some(&mut opts))?;
        let has_changes = !statuses.is_empty();

        let changed_files: Vec<String> = statuses
            .iter()
            .filter_map(|entry| {
                entry.path().map(|path| {
                    let status = entry.status();
                    let prefix = if status.is_wt_new() || status.is_index_new() {
                        "?? "
                    } else if status.is_wt_modified() || status.is_index_modified() {
                        " M "
                    } else if status.is_wt_deleted() || status.is_index_deleted() {
                        " D "
                    } else if status.is_wt_renamed() || status.is_index_renamed() {
                        " R "
                    } else {
                        "   "
                    };
                    format!("{}{}", prefix, path)
                })
            })
            .collect();

        Ok((has_changes, current_branch, changed_files))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_test_repo(temp_dir: &TempDir) -> std::path::PathBuf {
        let repo_path = temp_dir.path().join("test_repo");
        fs::create_dir_all(&repo_path).unwrap();

        // Initialize git repository
        Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to initialize git repo");

        // Configure git user for tests (local to this repo)
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        repo_path
    }

    #[test]
    fn test_get_status_empty_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir);

        let result = GitStatus::get_repository_status(&repo_path);
        assert!(result.is_ok());

        let (has_changes, branch, files) = result.unwrap();
        assert!(!has_changes);
        assert!(branch.is_none()); // No commits yet, so no branch
        assert!(files.is_empty());
    }

    #[test]
    fn test_get_status_with_initial_commit() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir);

        // Create and commit a file
        let test_file = repo_path.join("README.md");
        fs::write(&test_file, "# Test Repository").unwrap();

        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        let result = GitStatus::get_repository_status(&repo_path);
        assert!(result.is_ok());

        let (has_changes, branch, files) = result.unwrap();
        assert!(!has_changes); // No uncommitted changes
        assert_eq!(branch, Some("main".to_string()));
        assert!(files.is_empty());
    }

    #[test]
    fn test_get_status_with_changes() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = create_test_repo(&temp_dir);

        // Create initial commit
        let readme_file = repo_path.join("README.md");
        fs::write(&readme_file, "# Test Repository").unwrap();

        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Create new untracked file
        let new_file = repo_path.join("new_file.txt");
        fs::write(&new_file, "New content").unwrap();

        // Modify existing file
        fs::write(&readme_file, "# Modified Repository").unwrap();

        let result = GitStatus::get_repository_status(&repo_path);
        assert!(result.is_ok());

        let (has_changes, branch, files) = result.unwrap();
        assert!(has_changes);
        assert_eq!(branch, Some("main".to_string()));
        assert_eq!(files.len(), 2);

        // Check that we have both modified and new files
        let has_modified = files.iter().any(|f| f.contains("README.md"));
        let has_new = files.iter().any(|f| f.contains("new_file.txt"));
        assert!(has_modified);
        assert!(has_new);
    }

    #[test]
    fn test_get_status_invalid_repo() {
        let temp_dir = TempDir::new().unwrap();
        let non_repo_path = temp_dir.path().join("not_a_repo");
        fs::create_dir_all(&non_repo_path).unwrap();

        let result = GitStatus::get_repository_status(&non_repo_path);
        assert!(result.is_err());
    }
}
