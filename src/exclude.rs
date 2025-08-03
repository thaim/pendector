use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;

#[derive(Debug)]
pub struct ExcludeFilter {
    gitignore: Option<Gitignore>,
}

impl ExcludeFilter {
    /// 新しいExcludeFilterを作成する
    pub fn new() -> Self {
        Self { gitignore: None }
    }

    /// パターンリストからExcludeFilterを作成する
    pub fn from_patterns(patterns: &[String]) -> Result<Self, ignore::Error> {
        if patterns.is_empty() {
            return Ok(Self::new());
        }

        let mut builder = GitignoreBuilder::new("");

        for pattern in patterns {
            builder.add_line(None, pattern)?;
        }

        let gitignore = builder.build()?;
        Ok(Self {
            gitignore: Some(gitignore),
        })
    }

    /// 指定されたパスが除外対象かどうかを判定する
    pub fn is_excluded<P: AsRef<Path>>(&self, path: P) -> bool {
        if let Some(ref gitignore) = self.gitignore {
            matches!(
                gitignore.matched(path.as_ref(), path.as_ref().is_dir()),
                ignore::Match::Ignore(_)
            )
        } else {
            false
        }
    }

    /// 複数のパターンリストをマージして新しいExcludeFilterを作成する
    pub fn merge_patterns(pattern_lists: &[&[String]]) -> Result<Self, ignore::Error> {
        let merged_patterns: Vec<String> = pattern_lists
            .iter()
            .flat_map(|patterns| patterns.iter().cloned())
            .collect();

        Self::from_patterns(&merged_patterns)
    }
}

impl Default for ExcludeFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_exclude_filter_new() {
        let filter = ExcludeFilter::new();
        assert!(!filter.is_excluded("any/path"));
    }

    #[test]
    fn test_exclude_filter_empty_patterns() {
        let filter = ExcludeFilter::from_patterns(&[]).unwrap();
        assert!(!filter.is_excluded("any/path"));
    }

    #[test]
    fn test_exclude_filter_simple_pattern() {
        let patterns = vec!["node_modules".to_string()];
        let filter = ExcludeFilter::from_patterns(&patterns).unwrap();

        assert!(filter.is_excluded("node_modules"));
        assert!(filter.is_excluded("project/node_modules"));
        assert!(!filter.is_excluded("some_modules"));
    }

    #[test]
    fn test_exclude_filter_wildcard_pattern() {
        let patterns = vec!["**/target/**".to_string(), "target".to_string()];
        let filter = ExcludeFilter::from_patterns(&patterns).unwrap();

        assert!(filter.is_excluded("target"));
        assert!(filter.is_excluded("project/target"));
        assert!(filter.is_excluded("deep/nested/target"));
        assert!(filter.is_excluded("project/target/debug"));
        assert!(!filter.is_excluded("targeting"));
    }

    #[test]
    fn test_exclude_filter_multiple_patterns() {
        let patterns = vec![
            "node_modules".to_string(),
            "*.log".to_string(),
            "**/build/**".to_string(),
            "build".to_string(),
        ];
        let filter = ExcludeFilter::from_patterns(&patterns).unwrap();

        assert!(filter.is_excluded("node_modules"));
        assert!(filter.is_excluded("app.log"));
        assert!(filter.is_excluded("project/build"));
        assert!(filter.is_excluded("build"));
        assert!(!filter.is_excluded("src/main.rs"));
    }

    #[test]
    fn test_exclude_filter_merge_patterns() {
        let patterns1 = vec!["node_modules".to_string()];
        let patterns2 = vec!["target".to_string(), "*.tmp".to_string()];
        let pattern_lists = vec![&patterns1[..], &patterns2[..]];

        let filter = ExcludeFilter::merge_patterns(&pattern_lists).unwrap();

        assert!(filter.is_excluded("node_modules"));
        assert!(filter.is_excluded("target"));
        assert!(filter.is_excluded("file.tmp"));
        assert!(!filter.is_excluded("src/main.rs"));
    }

    #[test]
    fn test_exclude_filter_directory_vs_file() {
        let patterns = vec!["build".to_string()];
        let filter = ExcludeFilter::from_patterns(&patterns).unwrap();

        // Both files and directories named "build" should be excluded
        assert!(filter.is_excluded(PathBuf::from("build")));
        assert!(filter.is_excluded(PathBuf::from("project/build")));
    }
}
