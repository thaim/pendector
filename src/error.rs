use std::fmt;

/// Pendectorのカスタムエラー型
#[derive(Debug)]
pub enum PendectorError {
    /// Gitリポジトリが見つからない
    GitRepositoryNotFound(String),
    /// Gitリポジトリの操作に失敗
    GitOperationFailed {
        repo_path: String,
        operation: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// ファイルシステム操作に失敗
    FileSystemError {
        path: std::path::PathBuf,
        message: String,
    },
    /// パスが無効
    InvalidPath(String),
    /// ディレクトリスキャンに失敗
    ScanError {
        path: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// フォーマット処理に失敗
    FormatError(String),
    /// 設定エラー
    ConfigError {
        path: std::path::PathBuf,
        message: String,
    },
    /// ネットワークエラー（fetch関連）
    NetworkError { repo_path: String, message: String },
    /// タイムアウトエラー
    TimeoutError {
        repo_path: String,
        timeout_secs: u64,
    },
    /// 認証エラー
    AuthenticationError { repo_path: String, message: String },
}

impl fmt::Display for PendectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PendectorError::GitRepositoryNotFound(path) => {
                write!(f, "Git repository not found at '{path}'")
            }
            PendectorError::GitOperationFailed {
                repo_path,
                operation,
                source,
            } => {
                write!(
                    f,
                    "Git operation '{operation}' failed in '{repo_path}': {source}"
                )
            }
            PendectorError::FileSystemError { path, message } => {
                write!(f, "File system error for '{}': {message}", path.display())
            }
            PendectorError::InvalidPath(path) => {
                write!(f, "Invalid path: '{path}'")
            }
            PendectorError::ScanError { path, source } => {
                write!(f, "Failed to scan directory '{path}': {source}")
            }
            PendectorError::FormatError(msg) => {
                write!(f, "Format error: {msg}")
            }
            PendectorError::ConfigError { path, message } => {
                write!(f, "Configuration error in '{}': {message}", path.display())
            }
            PendectorError::NetworkError { repo_path, message } => {
                write!(f, "Network error for '{repo_path}': {message}")
            }
            PendectorError::TimeoutError {
                repo_path,
                timeout_secs,
            } => {
                write!(
                    f,
                    "Operation timed out after {timeout_secs}s for '{repo_path}'"
                )
            }
            PendectorError::AuthenticationError { repo_path, message } => {
                write!(f, "Authentication error for '{repo_path}': {message}")
            }
        }
    }
}

impl std::error::Error for PendectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PendectorError::GitOperationFailed { source, .. } => Some(source.as_ref()),
            PendectorError::ScanError { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// Pendectorの結果型
pub type PendectorResult<T> = Result<T, PendectorError>;

/// よく使用されるエラー変換用のヘルパー関数
impl PendectorError {
    /// Git2エラーからPendectorErrorを作成
    pub fn from_git2_error(repo_path: String, operation: String, error: git2::Error) -> Self {
        PendectorError::GitOperationFailed {
            repo_path,
            operation,
            source: Box::new(error),
        }
    }

    /// IOエラーからPendectorErrorを作成
    pub fn from_io_error(path: std::path::PathBuf, message: String) -> Self {
        PendectorError::FileSystemError { path, message }
    }

    /// fetch関連のエラーを分類して適切なPendectorErrorを作成
    pub fn from_fetch_error(repo_path: String, stderr: &str, exit_code: Option<i32>) -> Self {
        let repo_name = std::path::Path::new(&repo_path)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| "unknown".into())
            .to_string();

        // タイムアウトの場合
        if exit_code == Some(124) {
            return PendectorError::TimeoutError {
                repo_path: repo_name,
                timeout_secs: 0, // 実際のタイムアウト値は呼び出し元で設定
            };
        }

        // エラーメッセージの内容で分類
        if stderr.contains("Repository not found") {
            PendectorError::NetworkError {
                repo_path: repo_name,
                message: "Remote repository not found".to_string(),
            }
        } else if stderr.contains("Could not read from remote")
            || stderr.contains("Authentication failed")
        {
            PendectorError::AuthenticationError {
                repo_path: repo_name,
                message: "Authentication or access denied".to_string(),
            }
        } else if stderr.contains("Network is unreachable") || stderr.contains("Temporary failure")
        {
            PendectorError::NetworkError {
                repo_path: repo_name,
                message: "Network error".to_string(),
            }
        } else if !stderr.trim().is_empty() {
            PendectorError::NetworkError {
                repo_path: repo_name,
                message: stderr.trim().to_string(),
            }
        } else {
            PendectorError::NetworkError {
                repo_path: repo_name,
                message: "Failed to fetch from remote".to_string(),
            }
        }
    }
}
