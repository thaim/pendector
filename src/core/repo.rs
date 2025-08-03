use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct Repository {
    pub path: PathBuf,
    pub name: String,
    pub has_changes: bool,
    pub current_branch: Option<String>,
    pub changed_files: Vec<String>,
}

impl Repository {
    /// パスから適切なリポジトリ情報を抽出する
    pub fn new(path: PathBuf) -> Self {
        let name = if let Some(file_name) = path.file_name() {
            file_name.to_string_lossy().to_string()
        } else {
            // For root path or current directory, get the absolute path and use its name
            std::env::current_dir()
                .ok()
                .and_then(|current| {
                    let abs_path = if path.is_absolute() {
                        path.clone()
                    } else {
                        current.join(&path)
                    };
                    abs_path.canonicalize().ok()
                })
                .and_then(|canonical| {
                    canonical
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                })
                .unwrap_or_else(|| path.to_string_lossy().to_string())
        };

        Self {
            path,
            name,
            has_changes: false,
            current_branch: None,
            changed_files: Vec::new(),
        }
    }

    /// Git情報を取得する
    pub fn with_git_info(
        mut self,
        has_changes: bool,
        branch: Option<String>,
        files: Vec<String>,
    ) -> Self {
        self.has_changes = has_changes;
        self.current_branch = branch;
        self.changed_files = files;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_repository_new_with_simple_path() {
        let path = PathBuf::from("/home/user/projects/my_project");
        let repo = Repository::new(path.clone());

        assert_eq!(repo.path, path);
        assert_eq!(repo.name, "my_project");
        assert!(!repo.has_changes);
        assert!(repo.current_branch.is_none());
        assert!(repo.changed_files.is_empty());
    }

    #[test]
    fn test_repository_new_with_current_directory_path() {
        // Test with "." path
        let path = PathBuf::from(".");
        let repo = Repository::new(path.clone());

        assert_eq!(repo.path, path);
        // Should extract name from current working directory
        let current_dir_name = env::current_dir()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert_eq!(repo.name, current_dir_name);
    }

    #[test]
    fn test_repository_new_with_relative_path() {
        let path = PathBuf::from("src/core");
        let repo = Repository::new(path.clone());

        assert_eq!(repo.path, path);
        assert_eq!(repo.name, "core");
    }

    #[test]
    fn test_repository_new_with_unicode_path() {
        let path = PathBuf::from("/home/user/プロジェクト");
        let repo = Repository::new(path.clone());

        assert_eq!(repo.path, path);
        assert_eq!(repo.name, "プロジェクト");
    }

    #[test]
    fn test_repository_with_git_info() {
        let path = PathBuf::from("/test/repo");
        let files = vec!["file1.txt".to_string(), "file2.rs".to_string()];

        let repo = Repository::new(path.clone()).with_git_info(
            true,
            Some("main".to_string()),
            files.clone(),
        );

        assert_eq!(repo.path, path);
        assert_eq!(repo.name, "repo");
        assert!(repo.has_changes);
        assert_eq!(repo.current_branch, Some("main".to_string()));
        assert_eq!(repo.changed_files, files);
    }

    #[test]
    fn test_repository_with_git_info_no_branch() {
        let path = PathBuf::from("/test/repo");

        let repo = Repository::new(path.clone()).with_git_info(false, None, Vec::new());

        assert!(!repo.has_changes);
        assert!(repo.current_branch.is_none());
        assert!(repo.changed_files.is_empty());
    }

    #[test]
    fn test_repository_clone() {
        let path = PathBuf::from("/test/repo");
        let files = vec!["file1.txt".to_string()];

        let repo1 = Repository::new(path.clone()).with_git_info(
            true,
            Some("develop".to_string()),
            files.clone(),
        );
        let repo2 = repo1.clone();

        assert_eq!(repo1.path, repo2.path);
        assert_eq!(repo1.name, repo2.name);
        assert_eq!(repo1.has_changes, repo2.has_changes);
        assert_eq!(repo1.current_branch, repo2.current_branch);
        assert_eq!(repo1.changed_files, repo2.changed_files);
    }

    #[test]
    fn test_repository_debug_format() {
        let path = PathBuf::from("/test/repo");
        let repo = Repository::new(path).with_git_info(
            true,
            Some("feature/test".to_string()),
            vec!["test.rs".to_string()],
        );

        let debug_str = format!("{repo:?}");
        assert!(debug_str.contains("Repository"));
        assert!(debug_str.contains("/test/repo"));
        assert!(debug_str.contains("repo"));
        assert!(debug_str.contains("true"));
        assert!(debug_str.contains("feature/test"));
    }
}
