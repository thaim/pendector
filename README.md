# pendector

[![CI](https://github.com/thaim/pendector/actions/workflows/ci.yml/badge.svg)](https://github.com/thaim/pendector/actions/workflows/ci.yml)

A CLI tool that scans your local Git repositories and detects forgotten commits, unpushed branches, and other pending changes. If you manage multiple repositories across different directories, pendector gives you a single command to find what needs attention before it gets lost.

## Features

- Scan multiple directories at once to find uncommitted changes and untracked files
- Show remote sync status for each repository, indicating whether a push or pull is needed
- Customize scan targets, depth, and exclusion patterns via a TOML configuration file
- Output results as JSON for integration with CI pipelines and scripts
- Send scan results to Slack via Incoming Webhook

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

## Slack Notification

pendector can post scan results to a Slack channel via Incoming Webhook.

### Setup

1. Create a Slack Incoming Webhook by following the [Slack documentation](https://api.slack.com/messaging/webhooks).
2. Add the webhook URL to your `~/.config/pendector/config.toml`:

```toml
[slack]
webhook_url = "https://hooks.slack.com/services/T00/B00/XXX"
```

### CLI Options

| Flag | Description |
|------|-------------|
| `--notify-slack` | Enable Slack notification after scanning |
| `--slack-webhook-url <URL>` | Specify webhook URL directly (overrides config) |
| `--slack-notify-always` | Send notification even when no changes are detected |

```bash
# Notify Slack using URL from config
$ pendector --notify-slack

# Notify Slack with an explicit webhook URL
$ pendector --notify-slack --slack-webhook-url https://hooks.slack.com/services/T00/B00/XXX

# Always send notification, even when no pending changes exist
$ pendector --notify-slack --slack-notify-always
```

### Cron Example

Use pendector with cron to receive daily Slack alerts about pending changes:

```
# Check for pending changes every day at 9:00 AM
0 9 * * * /usr/local/bin/pendector --notify-slack --fetch --changes-only
```

### Systemd Timer Example

You can also use a systemd user timer instead of cron.

Create `~/.config/systemd/user/pendector.service`:

```ini
[Unit]
Description=Scan Git repositories for pending changes

[Service]
Type=oneshot
ExecStart=/usr/local/bin/pendector --notify-slack --fetch --changes-only
```

Create `~/.config/systemd/user/pendector.timer`:

```ini
[Unit]
Description=Run pendector daily

[Timer]
OnCalendar=*-*-* 09:00:00

[Install]
WantedBy=timers.target
```

Enable and start the timer:

```bash
systemctl --user daemon-reload
systemctl --user enable --now pendector.timer
```

### Configuration Reference

All Slack settings can be configured under the `[slack]` section in `config.toml`:

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `webhook_url` | string | — | Slack Incoming Webhook URL |
| `notify_only_changes` | bool | `true` | Only send notification when changes are detected |
| `username` | string | — | Bot username displayed in Slack |
| `icon_emoji` | string | — | Bot icon emoji (e.g. `:git:`) |
| `channel` | string | — | Channel to post to (overrides webhook default) |

```toml
[slack]
webhook_url = "https://hooks.slack.com/services/T00/B00/XXX"
notify_only_changes = true
username = "pendector"
icon_emoji = ":git:"
channel = "#dev-alerts"
```

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
