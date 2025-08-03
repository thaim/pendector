use crate::error::{PendectorError, PendectorResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub defaults: DefaultConfig,

    #[serde(default)]
    pub path_configs: Vec<PathConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,

    #[serde(default)]
    pub fetch: bool,

    #[serde(default = "default_fetch_timeout")]
    pub fetch_timeout: u64,

    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default)]
    pub verbose: bool,

    #[serde(default)]
    pub changes_only: bool,

    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub path: String,
    pub max_depth: Option<usize>,
    pub fetch: Option<bool>,
    pub fetch_timeout: Option<u64>,
    pub format: Option<String>,
    pub verbose: Option<bool>,
    pub changes_only: Option<bool>,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            max_depth: default_max_depth(),
            fetch: false,
            fetch_timeout: default_fetch_timeout(),
            format: default_format(),
            verbose: false,
            changes_only: false,
            paths: vec![".".to_string()],
        }
    }
}

fn default_max_depth() -> usize {
    3
}

fn default_fetch_timeout() -> u64 {
    5
}

fn default_format() -> String {
    "text".to_string()
}

impl Config {
    /// 設定ファイルを読み込む
    pub fn load(config_path: Option<&Path>) -> PendectorResult<Self> {
        let config_file_path = if let Some(path) = config_path {
            // CLI で指定されたパス
            path.to_path_buf()
        } else {
            // デフォルトの設定ファイルパス
            Self::default_config_path()?
        };

        if !config_file_path.exists() {
            // 設定ファイルが存在しない場合はデフォルト設定を返す
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_file_path).map_err(|e| {
            PendectorError::FileSystemError {
                path: config_file_path.clone(),
                message: format!("Failed to read config file: {e}"),
            }
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| PendectorError::ConfigError {
            path: config_file_path,
            message: format!("Failed to parse config file: {e}"),
        })?;

        Ok(config)
    }

    /// デフォルトの設定ファイルパスを取得
    fn default_config_path() -> PendectorResult<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| PendectorError::ConfigError {
            path: PathBuf::new(),
            message: "Could not determine config directory".to_string(),
        })?;

        Ok(config_dir.join("pendector").join("config.toml"))
    }

    /// 指定されたパスに対する設定を取得
    pub fn get_path_config(&self, target_path: &str) -> PathConfigResolved {
        // パス固有の設定を検索
        let path_config = self
            .path_configs
            .iter()
            .find(|pc| Self::path_matches(&pc.path, target_path));

        PathConfigResolved {
            max_depth: path_config
                .and_then(|pc| pc.max_depth)
                .unwrap_or(self.defaults.max_depth),
            fetch: path_config
                .and_then(|pc| pc.fetch)
                .unwrap_or(self.defaults.fetch),
            fetch_timeout: path_config
                .and_then(|pc| pc.fetch_timeout)
                .unwrap_or(self.defaults.fetch_timeout),
            format: path_config
                .and_then(|pc| pc.format.clone())
                .unwrap_or_else(|| self.defaults.format.clone()),
            verbose: path_config
                .and_then(|pc| pc.verbose)
                .unwrap_or(self.defaults.verbose),
            changes_only: path_config
                .and_then(|pc| pc.changes_only)
                .unwrap_or(self.defaults.changes_only),
        }
    }

    /// パスマッチングロジック
    fn path_matches(config_path: &str, target_path: &str) -> bool {
        // チルダ展開
        let expanded_config_path = shellexpand::tilde(config_path);
        let expanded_target_path = shellexpand::tilde(target_path);

        // 正規化
        let config_canonical = Path::new(expanded_config_path.as_ref())
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(expanded_config_path.as_ref()));
        let target_canonical = Path::new(expanded_target_path.as_ref())
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(expanded_target_path.as_ref()));

        // 完全一致または親ディレクトリかチェック
        target_canonical == config_canonical || target_canonical.starts_with(&config_canonical)
    }

    /// デフォルトパスの取得
    pub fn get_default_paths(&self) -> &[String] {
        &self.defaults.paths
    }
}

#[derive(Debug, Clone)]
pub struct PathConfigResolved {
    pub max_depth: usize,
    pub fetch: bool,
    pub fetch_timeout: u64,
    pub format: String,
    pub verbose: bool,
    pub changes_only: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.defaults.max_depth, 3);
        assert!(!config.defaults.fetch);
        assert_eq!(config.defaults.fetch_timeout, 5);
        assert_eq!(config.defaults.format, "text");
        assert!(!config.defaults.verbose);
        assert!(!config.defaults.changes_only);
        assert_eq!(config.defaults.paths, vec!["."]);
    }

    #[test]
    fn test_load_config_file_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_path = temp_dir.path().join("non_existent.toml");

        let config = Config::load(Some(&non_existent_path)).unwrap();
        assert_eq!(config.defaults.max_depth, 3);
    }

    #[test]
    fn test_load_valid_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config_content = r#"
[defaults]
max_depth = 5
fetch = true
fetch_timeout = 10
format = "json"
verbose = true
changes_only = true
paths = ["~/src", "~/work"]

[[path_configs]]
path = "~/src"
max_depth = 7
fetch = true

[[path_configs]]
path = "~/work"
max_depth = 2
fetch = false
"#;

        std::fs::write(&config_path, config_content).unwrap();

        let config = Config::load(Some(&config_path)).unwrap();
        assert_eq!(config.defaults.max_depth, 5);
        assert!(config.defaults.fetch);
        assert_eq!(config.defaults.fetch_timeout, 10);
        assert_eq!(config.defaults.format, "json");
        assert!(config.defaults.verbose);
        assert!(config.defaults.changes_only);
        assert_eq!(config.defaults.paths, vec!["~/src", "~/work"]);

        assert_eq!(config.path_configs.len(), 2);
        assert_eq!(config.path_configs[0].path, "~/src");
        assert_eq!(config.path_configs[0].max_depth, Some(7));
        assert_eq!(config.path_configs[0].fetch, Some(true));
    }

    #[test]
    fn test_get_path_config_no_match() {
        let config = Config::default();
        let path_config = config.get_path_config("/some/path");

        assert_eq!(path_config.max_depth, 3);
        assert!(!path_config.fetch);
        assert_eq!(path_config.fetch_timeout, 5);
        assert_eq!(path_config.format, "text");
    }

    #[test]
    fn test_get_path_config_with_match() {
        let mut config = Config::default();
        config.path_configs.push(PathConfig {
            path: "/test/path".to_string(),
            max_depth: Some(10),
            fetch: Some(true),
            fetch_timeout: Some(20),
            format: Some("json".to_string()),
            verbose: Some(true),
            changes_only: Some(true),
        });

        let path_config = config.get_path_config("/test/path");
        assert_eq!(path_config.max_depth, 10);
        assert!(path_config.fetch);
        assert_eq!(path_config.fetch_timeout, 20);
        assert_eq!(path_config.format, "json");
        assert!(path_config.verbose);
        assert!(path_config.changes_only);
    }

    #[test]
    fn test_path_matches() {
        assert!(Config::path_matches("/test/path", "/test/path"));
        assert!(Config::path_matches("/test", "/test/subdir"));
        assert!(!Config::path_matches("/test/path", "/other/path"));
    }
}
