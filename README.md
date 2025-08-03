# pendector

[![CI](https://github.com/thaim/pendector/actions/workflows/ci.yml/badge.svg)](https://github.com/thaim/pendector/actions/workflows/ci.yml)

A CLI tool for detecting pending changes and remote sync status across your local Git repositories.

## Features

- 🔍 Detect uncommitted changes and untracked files
- 🔄 Check remote sync status (pull/push needed)
- ⚡ Auto-fetch option for accurate remote status
- 📋 JSON output for automation

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

### More options
Run `pendector --help` for all available options.

## Installation

### Option 1: Download pre-built binary (Recommended)

Download the latest binary from the [Releases page](https://github.com/thaim/pendector/releases).

### Option 2: Build from source

Requirements: Rust 1.70+ and Cargo

```bash
git clone https://github.com/thaim/pendector.git
cd pendector
cargo build --release
```
