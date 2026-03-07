use crate::core::Repository;
use crate::error::{PendectorError, PendectorResult};

pub struct SlackNotifier {
    webhook_url: String,
    username: Option<String>,
    icon_emoji: Option<String>,
    channel: Option<String>,
}

impl SlackNotifier {
    pub fn new(
        webhook_url: String,
        username: Option<String>,
        icon_emoji: Option<String>,
        channel: Option<String>,
    ) -> Self {
        Self {
            webhook_url,
            username,
            icon_emoji,
            channel,
        }
    }

    pub fn notify(&self, repositories: &[Repository]) -> PendectorResult<()> {
        let message = Self::format_message(repositories);
        self.send_webhook(&message)
    }

    pub fn format_message(repositories: &[Repository]) -> String {
        if repositories.is_empty() {
            return "*pendector: No repositories found*".to_string();
        }

        let changed_repos: Vec<&Repository> = repositories
            .iter()
            .filter(|r| r.has_changes || r.needs_push || r.needs_pull)
            .collect();

        if changed_repos.is_empty() {
            let count = repositories.len();
            return format!("*pendector: All repositories clean* ({count} repositories scanned)");
        }

        let count = changed_repos.len();
        let mut lines = vec![format!(
            "*pendector: {count} repositories with pending changes*"
        )];
        lines.push(String::new());

        let display_repos = if changed_repos.len() > 20 {
            &changed_repos[..20]
        } else {
            &changed_repos
        };

        for repo in display_repos {
            let branch = repo.current_branch.as_deref().unwrap_or("unknown");
            let file_count = repo.changed_files.len();
            let files_label = if file_count == 1 {
                "1 changed file".to_string()
            } else {
                format!("{file_count} changed files")
            };

            let remote_status = if repo.needs_push && repo.needs_pull {
                " [needs push and pull]"
            } else if repo.needs_push {
                " [needs push]"
            } else if repo.needs_pull {
                " [needs pull]"
            } else {
                ""
            };

            lines.push(format!(
                "\u{2022} *{}* (`{branch}`) - {files_label}{remote_status}",
                repo.name
            ));
        }

        if changed_repos.len() > 20 {
            let remaining = changed_repos.len() - 20;
            lines.push(format!("...and {remaining} more"));
        }

        lines.join("\n")
    }

    fn send_webhook(&self, message: &str) -> PendectorResult<()> {
        let mut payload = serde_json::json!({
            "text": message
        });

        if let Some(username) = &self.username {
            payload["username"] = serde_json::json!(username);
        }
        if let Some(icon_emoji) = &self.icon_emoji {
            payload["icon_emoji"] = serde_json::json!(icon_emoji);
        }
        if let Some(channel) = &self.channel {
            payload["channel"] = serde_json::json!(channel);
        }

        let body = serde_json::to_string(&payload).unwrap();
        ureq::post(&self.webhook_url)
            .header("Content-Type", "application/json")
            .send(body.as_bytes())
            .map_err(|e| PendectorError::SlackNotifyError {
                message: e.to_string(),
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_repo(
        name: &str,
        has_changes: bool,
        branch: &str,
        changed_files: Vec<&str>,
        needs_push: bool,
        needs_pull: bool,
    ) -> Repository {
        Repository {
            path: PathBuf::from(format!("/path/to/{name}")),
            name: name.to_string(),
            has_changes,
            current_branch: Some(branch.to_string()),
            changed_files: changed_files.into_iter().map(|s| s.to_string()).collect(),
            needs_pull,
            needs_push,
            remote_branch: Some(format!("origin/{branch}")),
        }
    }

    #[test]
    fn test_format_message_empty() {
        let result = SlackNotifier::format_message(&[]);
        assert_eq!(result, "*pendector: No repositories found*");
    }

    #[test]
    fn test_format_message_no_changes() {
        let repos = vec![
            make_repo("repo1", false, "main", vec![], false, false),
            make_repo("repo2", false, "main", vec![], false, false),
        ];
        let result = SlackNotifier::format_message(&repos);
        assert_eq!(
            result,
            "*pendector: All repositories clean* (2 repositories scanned)"
        );
    }

    #[test]
    fn test_format_message_with_changes() {
        let repos = vec![make_repo(
            "my-project",
            true,
            "main",
            vec!["M src/main.rs", "M src/lib.rs"],
            false,
            false,
        )];
        let result = SlackNotifier::format_message(&repos);
        assert!(result.contains("*pendector: 1 repositories with pending changes*"));
        assert!(result.contains("\u{2022} *my-project* (`main`) - 2 changed files"));
    }

    #[test]
    fn test_format_message_mixed() {
        let repos = vec![
            make_repo(
                "changed-repo",
                true,
                "main",
                vec!["M file.rs"],
                false,
                false,
            ),
            make_repo("clean-repo", false, "main", vec![], false, false),
        ];
        let result = SlackNotifier::format_message(&repos);
        assert!(result.contains("*pendector: 1 repositories with pending changes*"));
        assert!(result.contains("*changed-repo*"));
        assert!(!result.contains("*clean-repo*"));
    }

    #[test]
    fn test_format_message_with_remote_status() {
        let repos = vec![
            make_repo("push-repo", true, "main", vec!["M file.rs"], true, false),
            make_repo("pull-repo", true, "dev", vec!["M file.rs"], false, true),
            make_repo("both-repo", true, "feature", vec!["M file.rs"], true, true),
        ];
        let result = SlackNotifier::format_message(&repos);
        assert!(result.contains("*push-repo* (`main`) - 1 changed file [needs push]"));
        assert!(result.contains("*pull-repo* (`dev`) - 1 changed file [needs pull]"));
        assert!(result.contains("*both-repo* (`feature`) - 1 changed file [needs push and pull]"));
    }

    #[test]
    fn test_format_message_single_changed_file() {
        let repos = vec![make_repo(
            "repo",
            true,
            "main",
            vec!["M src/main.rs"],
            false,
            false,
        )];
        let result = SlackNotifier::format_message(&repos);
        assert!(result.contains("1 changed file"));
        assert!(!result.contains("1 changed files"));
    }

    #[test]
    fn test_format_message_truncation() {
        let repos: Vec<Repository> = (0..25)
            .map(|i| {
                make_repo(
                    &format!("repo-{i}"),
                    true,
                    "main",
                    vec!["M file.rs"],
                    false,
                    false,
                )
            })
            .collect();
        let result = SlackNotifier::format_message(&repos);
        assert!(result.contains("*pendector: 25 repositories with pending changes*"));
        assert!(result.contains("...and 5 more"));
        // 21番目のリポジトリ(repo-20)は表示されない
        assert!(!result.contains("*repo-20*"));
    }
}
