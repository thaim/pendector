use crate::core::Repository;
use colored::*;
use serde_json;

pub struct OutputFormatter {
    pub verbose: bool,
    pub format: String,
}

impl OutputFormatter {
    pub fn new(verbose: bool, format: String) -> Self {
        Self { verbose, format }
    }

    pub fn format_repositories(&self, repositories: &[Repository]) -> String {
        match self.format.as_str() {
            "json" => self.format_repositories_json(repositories),
            _ => self.format_repositories_text(repositories),
        }
    }

    fn format_repositories_json(&self, repositories: &[Repository]) -> String {
        serde_json::to_string_pretty(repositories).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_repositories_text(&self, repositories: &[Repository]) -> String {
        if repositories.is_empty() {
            return "No repositories found.".to_string();
        }

        let mut output = String::new();

        // Add header with summary
        let total_count = repositories.len();
        let changed_count = repositories.iter().filter(|r| r.has_changes).count();

        output.push_str(&format!("Found {total_count} repositories"));
        if changed_count > 0 {
            output.push_str(&format!(" ({changed_count} with changes)"));
        }
        output.push_str(":\n\n");

        for repo in repositories {
            output.push_str(&self.format_repository(repo));
            output.push('\n');
        }

        output
    }

    fn format_repository(&self, repo: &Repository) -> String {
        let name = if repo.has_changes {
            repo.name.red().to_string()
        } else {
            repo.name.green().to_string()
        };

        let branch = repo.current_branch.as_deref().unwrap_or("unknown");
        let files_count = repo.changed_files.len();
        let path = repo
            .path
            .canonicalize()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| repo.path.display().to_string());

        if self.verbose {
            // Verbose mode shows additional details like specific changed files
            let mut result =
                format!("{name} [{branch}] ({files_count} changed files)\n  Path: {path}");

            if !repo.changed_files.is_empty() {
                result.push_str("\n  Changed files:");
                for file in &repo.changed_files {
                    result.push_str(&format!("\n    {file}"));
                }
            }
            result
        } else {
            // Default mode shows essential information
            format!("{name} [{branch}] ({files_count} changed files) - {path}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_repository(
        name: &str,
        has_changes: bool,
        branch: Option<&str>,
        file_count: usize,
    ) -> Repository {
        let files = if file_count > 0 {
            (0..file_count).map(|i| format!("file{i}.txt")).collect()
        } else {
            Vec::new()
        };

        Repository::new(PathBuf::from(format!("/test/{name}"))).with_git_info(
            has_changes,
            branch.map(|s| s.to_string()),
            files,
        )
    }

    #[test]
    fn test_format_repositories_empty() {
        let formatter = OutputFormatter::new(false, "text".to_string());
        let repositories = Vec::new();

        let result = formatter.format_repositories(&repositories);
        assert_eq!(result, "No repositories found.");
    }

    #[test]
    fn test_format_repositories_single_clean() {
        let formatter = OutputFormatter::new(false, "text".to_string());
        let repositories = vec![create_test_repository("clean_repo", false, Some("main"), 0)];

        let result = formatter.format_repositories(&repositories);
        // Should contain the repo name, but we can't easily test colors in unit tests
        assert!(result.contains("clean_repo"));
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_format_repositories_single_with_changes() {
        let formatter = OutputFormatter::new(false, "text".to_string());
        let repositories = vec![create_test_repository(
            "dirty_repo",
            true,
            Some("develop"),
            3,
        )];

        let result = formatter.format_repositories(&repositories);
        assert!(result.contains("dirty_repo"));
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_format_repositories_multiple() {
        let formatter = OutputFormatter::new(false, "text".to_string());
        let repositories = vec![
            create_test_repository("repo1", false, Some("main"), 0),
            create_test_repository("repo2", true, Some("feature"), 2),
            create_test_repository("repo3", false, None, 0),
        ];

        let result = formatter.format_repositories(&repositories);
        assert!(result.contains("repo1"));
        assert!(result.contains("repo2"));
        assert!(result.contains("repo3"));

        // Should have 5 lines: header (2 lines) + 3 repo lines
        assert_eq!(result.lines().count(), 5);
    }

    #[test]
    fn test_format_repositories_verbose_mode() {
        let formatter = OutputFormatter::new(true, "text".to_string());
        let repositories = vec![
            create_test_repository("repo1", false, Some("main"), 0),
            create_test_repository("repo2", true, Some("feature/test"), 5),
            create_test_repository("repo3", true, None, 2),
        ];

        let result = formatter.format_repositories(&repositories);

        // Should contain branch information
        assert!(result.contains("[main]"));
        assert!(result.contains("[feature/test]"));
        assert!(result.contains("[unknown]")); // For repo3 with no branch

        // Should contain file count information
        assert!(result.contains("(0 changed files)"));
        assert!(result.contains("(5 changed files)"));
        assert!(result.contains("(2 changed files)"));
    }

    #[test]
    fn test_format_repository_default_mode() {
        let formatter = OutputFormatter::new(false, "text".to_string());
        let repo = create_test_repository("test_repo", true, Some("main"), 3);

        let result = formatter.format_repository(&repo);
        // Now default mode shows all essential information
        assert!(result.contains("test_repo"));
        assert!(result.contains("[main]"));
        assert!(result.contains("changed files"));
    }

    #[test]
    fn test_format_repository_verbose_mode() {
        let formatter = OutputFormatter::new(true, "text".to_string());
        let repo = create_test_repository("test_repo", true, Some("develop"), 7);

        let result = formatter.format_repository(&repo);
        assert!(result.contains("test_repo"));
        assert!(result.contains("[develop]"));
        assert!(result.contains("(7 changed files)"));
    }

    #[test]
    fn test_format_repository_verbose_no_branch() {
        let formatter = OutputFormatter::new(true, "text".to_string());
        let repo = create_test_repository("test_repo", false, None, 0);

        let result = formatter.format_repository(&repo);
        assert!(result.contains("test_repo"));
        assert!(result.contains("[unknown]"));
        assert!(result.contains("(0 changed files)"));
    }

    #[test]
    fn test_formatter_verbose_flag() {
        let verbose_formatter = OutputFormatter::new(true, "text".to_string());
        let simple_formatter = OutputFormatter::new(false, "text".to_string());

        assert!(verbose_formatter.verbose);
        assert!(!simple_formatter.verbose);
    }

    #[test]
    fn test_format_repositories_json() {
        let formatter = OutputFormatter::new(false, "json".to_string());
        let repositories = vec![
            create_test_repository("repo1", false, Some("main"), 0),
            create_test_repository("repo2", true, Some("feature"), 2),
        ];

        let result = formatter.format_repositories(&repositories);

        // Should be valid JSON
        assert!(serde_json::from_str::<Vec<serde_json::Value>>(&result).is_ok());

        // Should contain repository data
        assert!(result.contains("repo1"));
        assert!(result.contains("repo2"));
        assert!(result.contains("main"));
        assert!(result.contains("feature"));
    }

    #[test]
    fn test_format_repositories_json_empty() {
        let formatter = OutputFormatter::new(false, "json".to_string());
        let repositories = Vec::new();

        let result = formatter.format_repositories(&repositories);
        assert_eq!(result, "[]");
    }
}
