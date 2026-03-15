# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**pendector** is a Rust CLI tool for detecting and managing pending changes across local Git repositories. It scans directory trees, identifies Git repositories with uncommitted changes or unpushed/unpulled commits, and reports status in text or JSON format. Features include parallel scanning, TOML-based configuration, per-path settings, gitignore-style exclusion patterns, and remote sync detection.

## Development Commands

### Build
```bash
cargo build
```

### Run
```bash
cargo run -- [arguments]
```

### Test
```bash
cargo test
```

### Run single test
```bash
cargo test test_name
```

### Check (faster than build, for linting)
```bash
cargo check
```

### Format code
```bash
cargo fmt
```

### Lint with Clippy
```bash
cargo clippy
```

## Architecture

### Module Structure

```
src/
├── main.rs              # Entry point: CLI解析 → 設定読み込み → スキャン → フィルタ → 出力
├── lib.rs               # ライブラリエクスポート
├── config.rs            # TOML設定ファイル管理（Config, DefaultConfig, PathConfig）
├── error.rs             # カスタムエラー型（PendectorError enum）
├── exclude.rs           # 除外フィルタ（ignoreクレートによるgitignoreスタイルマッチング）
├── cli/
│   └── mod.rs           # CLI引数定義（Args struct, clap derive）
├── core/
│   ├── mod.rs
│   ├── repo.rs          # Repositoryデータ構造（ビルダーパターン、Serialize対応）
│   └── scanner.rs       # リポジトリ探索（walkdir + rayon並列処理）
├── git/
│   ├── mod.rs
│   └── status.rs        # Gitステータス検出・fetch操作（git2クレート）
├── output/
│   ├── mod.rs
│   └── formatter.rs     # テキスト/JSON出力フォーマット（colored出力対応）
└── notify/
    ├── mod.rs
    └── slack.rs         # Slack通知（Incoming Webhook経由、ureqクレート）
```

### Main Flow

1. CLI引数解析（`Args::parse()`）
2. 設定ファイル読み込み（`Config::load()`、TOML形式）
3. スキャン対象パス決定（CLI引数 > 設定ファイル > カレントディレクトリ）
4. パス別設定の解決（CLI引数が設定ファイルをオーバーライド）
5. 除外パターンのマージ（CLI引数 > パス別設定 > デフォルト設定）
6. リポジトリスキャン（`scanner.scan_with_exclude()`、rayon並列処理）
7. 結果フィルタ（`--changes-only` で変更ありのみ）
8. 出力フォーマット（テキスト or JSON）
9. Slack通知（`--notify-slack` 指定時、`SlackNotifier`）

## Dependencies

### Runtime
- **clap** (derive): CLI引数解析
- **walkdir**: 再帰的ディレクトリ走査
- **colored**: ターミナルカラー出力
- **git2**: Git操作（ステータス検出、リモート同期確認）
- **serde** (derive) + **serde_json**: シリアライズ/JSON出力
- **rayon**: 並列処理（スキャン・Git操作）
- **indicatif**: プログレスバー表示
- **toml**: TOML設定ファイルパース
- **dirs**: プラットフォーム固有ディレクトリ取得
- **shellexpand**: シェル変数展開（~, $VAR）
- **ignore**: gitignore互換パターンマッチング
- **ureq**: HTTPクライアント（Slack Webhook送信）

### Dev
- **assert_cmd**: CLIテストフレームワーク
- **tempfile**: テスト用一時ファイル/ディレクトリ
- **predicates**: テスト用アサーション述語

## Testing Strategy

- **Unit tests**: 各モジュール内に `#[cfg(test)]` で配置（config, exclude, repo, scanner, status, formatter）
- **Integration tests**: `tests/cli.rs` で `assert_cmd::Command` によるバイナリ直接テスト
- **テストパターン**: `tempfile::TempDir` で一時ディレクトリ作成、`predicates::str` で出力検証、モック用 `.git/` ディレクトリ生成

## Code Quality Rules

ALWAYS run the following after making code changes:
1. `cargo fmt` - Auto-format code
2. `cargo test` - Run all tests
3. `cargo clippy` - Run linter (if available)

These commands must be executed before considering any task complete.

@docs/RUST_PRACTICES.md
