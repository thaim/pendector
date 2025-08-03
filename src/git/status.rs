use git2::{Repository as Git2Repository, StatusOptions};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RepoStatus {
    pub has_changes: bool,
    pub current_branch: Option<String>,
    pub changed_files: Vec<String>,
    pub needs_pull: bool,
    pub needs_push: bool,
    pub remote_branch: Option<String>,
}

pub struct GitStatus;

impl GitStatus {
    /// git2ライブラリを使用してリポジトリの状態を取得
    pub fn get_repository_status<P: AsRef<Path>>(
        repo_path: P,
    ) -> Result<RepoStatus, Box<dyn std::error::Error>> {
        Self::get_repository_status_with_fetch(repo_path, false)
    }

    /// git2ライブラリを使用してリポジトリの状態を取得（fetch実行オプション付き）
    pub fn get_repository_status_with_fetch<P: AsRef<Path>>(
        repo_path: P,
        should_fetch: bool,
    ) -> Result<RepoStatus, Box<dyn std::error::Error>> {
        let repo_path = repo_path.as_ref();

        // git2でリポジトリを開く
        let repo = Git2Repository::open(repo_path)?;

        // fetchが要求された場合は実行
        if should_fetch {
            Self::perform_fetch(repo_path)?;
        }

        // 現在のブランチ名を取得
        let current_branch = if let Ok(head) = repo.head() {
            head.shorthand().map(|name| name.to_string())
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
                    format!("{prefix}{path}")
                })
            })
            .collect();

        // リモート同期状態の確認
        let (needs_pull, needs_push, remote_branch) = Self::check_remote_sync(&repo)?;

        Ok(RepoStatus {
            has_changes,
            current_branch,
            changed_files,
            needs_pull,
            needs_push,
            remote_branch,
        })
    }

    /// リモートブランチとの同期状態をチェック
    fn check_remote_sync(
        repo: &Git2Repository,
    ) -> Result<(bool, bool, Option<String>), Box<dyn std::error::Error>> {
        // デフォルト値
        let mut needs_pull = false;
        let mut needs_push = false;
        let mut remote_branch = None;

        // 現在のHEADを取得
        if let Ok(head) = repo.head() {
            if let Some(branch_name) = head.shorthand() {
                // 対応するリモートブランチを検索
                let remote_branch_name = format!("origin/{branch_name}");

                if let Some(local_oid) = head.target() {
                    // リモートブランチの存在確認とOID取得
                    if let Ok(remote_ref) =
                        repo.find_reference(&format!("refs/remotes/{remote_branch_name}"))
                    {
                        remote_branch = Some(remote_branch_name.clone());

                        if let Some(remote_oid) = remote_ref.target() {
                            // ローカルとリモートのOIDが異なる場合の詳細チェック
                            if local_oid != remote_oid {
                                // git merge-base を使ってコミットの関係性を確認
                                match repo.merge_base(local_oid, remote_oid) {
                                    Ok(base_oid) => {
                                        // リモートの方が進んでいる（pull必要）
                                        if base_oid == local_oid && base_oid != remote_oid {
                                            needs_pull = true;
                                        }
                                        // ローカルの方が進んでいる（push必要）
                                        else if base_oid == remote_oid && base_oid != local_oid {
                                            needs_push = true;
                                        }
                                        // 分岐している（両方必要）
                                        else if base_oid != local_oid && base_oid != remote_oid {
                                            needs_pull = true;
                                            needs_push = true;
                                        }
                                    }
                                    Err(_) => {
                                        // merge-baseが見つからない場合は分岐とみなす
                                        needs_pull = true;
                                        needs_push = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((needs_pull, needs_push, remote_branch))
    }

    /// git fetchを実行してリモートの最新状態を取得
    fn perform_fetch<P: AsRef<Path>>(repo_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let repo_path = repo_path.as_ref();

        // git fetch コマンドを実行（非対話的モード）
        let output = Command::new("git")
            .args(["fetch", "--all", "--quiet"])
            .env("GIT_TERMINAL_PROMPT", "0") // ターミナルプロンプトを無効化
            .env("GIT_ASKPASS", "true") // 認証プロンプトを無効化（常にfalseを返す）
            .env("SSH_ASKPASS", "true") // SSH認証プロンプトも無効化
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let repo_name = repo_path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_else(|| "unknown".into());

            // ネットワークエラーや認証エラーは警告として扱い、処理を継続
            if stderr.contains("Repository not found") {
                eprintln!("Warning: {repo_name}: Remote repository not found (skipping fetch)");
            } else if stderr.contains("Could not read from remote")
                || stderr.contains("Authentication failed")
            {
                eprintln!("Warning: {repo_name}: Authentication or access denied (skipping fetch)");
            } else if stderr.contains("Network is unreachable")
                || stderr.contains("Temporary failure")
            {
                eprintln!("Warning: {repo_name}: Network error (skipping fetch)");
            } else if !stderr.trim().is_empty() {
                eprintln!("Warning: {repo_name}: Failed to fetch - {}", stderr.trim());
            } else {
                eprintln!("Warning: {repo_name}: Failed to fetch from remote");
            }
        }

        Ok(())
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

        // Set default branch name to 'main' for consistency across environments
        Command::new("git")
            .args(["config", "init.defaultBranch", "main"])
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

        let status = result.unwrap();
        assert!(!status.has_changes);
        assert!(status.current_branch.is_none()); // No commits yet, so no branch
        assert!(status.changed_files.is_empty());
        assert!(!status.needs_pull);
        assert!(!status.needs_push);
        assert!(status.remote_branch.is_none());
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

        // Rename branch to 'main' if it's not already 'main'
        Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        let result = GitStatus::get_repository_status(&repo_path);
        assert!(result.is_ok());

        let status = result.unwrap();
        assert!(!status.has_changes); // No uncommitted changes
        assert_eq!(status.current_branch, Some("main".to_string()));
        assert!(status.changed_files.is_empty());
        assert!(!status.needs_pull); // No remote configured
        assert!(!status.needs_push); // No remote configured
        assert!(status.remote_branch.is_none());
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

        // Rename branch to 'main' if it's not already 'main'
        Command::new("git")
            .args(["branch", "-M", "main"])
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

        let status = result.unwrap();
        assert!(status.has_changes);
        assert_eq!(status.current_branch, Some("main".to_string()));
        assert_eq!(status.changed_files.len(), 2);

        // Check that we have both modified and new files
        let has_modified = status.changed_files.iter().any(|f| f.contains("README.md"));
        let has_new = status
            .changed_files
            .iter()
            .any(|f| f.contains("new_file.txt"));
        assert!(has_modified);
        assert!(has_new);

        // Remote sync status for local-only repo
        assert!(!status.needs_pull);
        assert!(!status.needs_push);
        assert!(status.remote_branch.is_none());
    }

    #[test]
    fn test_get_status_invalid_repo() {
        let temp_dir = TempDir::new().unwrap();
        let non_repo_path = temp_dir.path().join("not_a_repo");
        fs::create_dir_all(&non_repo_path).unwrap();

        let result = GitStatus::get_repository_status(&non_repo_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_status_with_remote_info() {
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

        Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        let result = GitStatus::get_repository_status(&repo_path);
        assert!(result.is_ok());

        let status = result.unwrap();
        assert!(!status.has_changes);
        assert_eq!(status.current_branch, Some("main".to_string()));
        assert!(status.changed_files.is_empty());

        // For a local-only repository without remote, these should be false
        assert!(!status.needs_pull);
        assert!(!status.needs_push);
        assert!(status.remote_branch.is_none());
    }

    #[test]
    fn test_get_status_with_fetch_option() {
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

        Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Test with fetch option (should not fail for local repo)
        let result = GitStatus::get_repository_status_with_fetch(&repo_path, true);
        assert!(result.is_ok());

        let status = result.unwrap();
        assert!(!status.has_changes);
        assert_eq!(status.current_branch, Some("main".to_string()));
        assert!(status.changed_files.is_empty());

        // For a local-only repository, fetch should not affect the result
        assert!(!status.needs_pull);
        assert!(!status.needs_push);
        assert!(status.remote_branch.is_none());
    }
}
