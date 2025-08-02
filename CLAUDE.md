# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**pendector** is a Rust CLI tool for detecting and managing pending changes across local repositories. The project uses Cargo for dependency management and clap for command-line argument parsing.

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

- **src/main.rs**: Entry point with clap-based CLI argument parsing. Currently structured as a simple echo-like utility but intended for repository management
- **tests/cli.rs**: Integration tests using `assert_cmd` crate for CLI testing
- **Cargo.toml**: Project configuration with clap dependency for CLI functionality

## Dependencies

- **clap v4.5.17**: Command-line argument parsing with derive features
- **assert_cmd** (dev dependency): CLI testing framework

## Testing Strategy

Integration tests are located in `tests/` directory and use `assert_cmd::Command` to test the binary directly. Tests verify both success and failure cases.

## Code Quality Rules

ALWAYS run the following after making code changes:
1. `cargo fmt` - Auto-format code
2. `cargo test` - Run all tests
3. `cargo clippy` - Run linter (if available)

These commands must be executed before considering any task complete.