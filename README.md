# pendector

[![CI](https://github.com/thaim/pendector/actions/workflows/ci.yml/badge.svg)](https://github.com/thaim/pendector/actions/workflows/ci.yml)

A CLI tool that scans your local Git repositories and detects forgotten commits, unpushed branches, and other pending changes. If you manage multiple repositories across different directories, pendector gives you a single command to find what needs attention before it gets lost.

## Features

- Scan multiple directories at once to find uncommitted changes and untracked files
- Show remote sync status for each repository, indicating whether a push or pull is needed
- Customize scan targets, depth, and exclusion patterns via a TOML configuration file
- Output results as JSON for integration with CI pipelines and scripts

## Quick Start

### Basic scan

```bash
$ pendector
Found 2 repositories (1 with changes):

my-project [main] [↑] (2 changed files) - /home/user/projects/my-project
web-app [develop] (0 changed files) - /home/user/projects/web-app
```

### Detailed information

```bash
$ pendector -v
Found 2 repositories (1 with changes):

my-project [main] [↑] (2 changed files)
  Path: /home/user/projects/my-project
  Remote: origin/main
  Sync status: needs push
  Changed files:
     M src/main.rs
    ?? README.md
```

### Fetch latest remote status

```bash
$ pendector --fetch
```

### Show only repositories with changes

```bash
$ pendector -c
```

### Scan specific directories

```bash
$ pendector ~/src ~/work
```

### More options

Run `pendector --help` for all available options.

## Configuration

pendector loads its configuration from `~/.config/pendector/config.toml`.

```toml
[defaults]
paths = ["~/src", "~/work"]
fetch = true
changes_only = true
exclude_patterns = ["node_modules", ".archive"]

[[path_configs]]
path = "~/work"
max_depth = 2
fetch = false
```

CLI options override config file settings. Run `pendector --help` for all available options.

## Installation

### Option 1: Download pre-built binary (Recommended)

Download the latest binary from the [Releases page](https://github.com/thaim/pendector/releases).

Pre-built binaries are available for Linux (x86_64) and macOS (Apple Silicon).

### Option 2: Build from source

Requirements: Rust 1.70+ and Cargo

```bash
git clone https://github.com/thaim/pendector.git
cd pendector
cargo build --release
```

## License

MIT
