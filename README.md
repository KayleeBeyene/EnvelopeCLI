# EnvelopeCLI

Terminal-based zero-based budgeting application inspired by YNAB.

## Overview

EnvelopeCLI is a command-line and terminal UI (TUI) budgeting application that implements zero-based budgeting principles. Every dollar gets a job, and you take control of your finances from the terminal.

## Features

- **Zero-based budgeting**: Assign every dollar to a category
- **Multiple budget periods**: Monthly, weekly, or bi-weekly budgeting
- **Account management**: Track checking, savings, credit cards, and more
- **Transaction tracking**: Manual entry and CSV import
- **Category management**: Organize spending into groups and categories
- **Reconciliation**: Match your records with bank statements
- **Reports**: Budget overview, spending analysis, net worth tracking
- **TUI Interface**: Full terminal user interface with keyboard navigation
- **CLI Commands**: Script-friendly command-line interface
- **Local storage**: Your data stays on your machine (JSON files)
- **Optional encryption**: AES-256-GCM encryption for sensitive data

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/KayleeBeyene/EnvelopeCLI.git
cd EnvelopeCLI

# Build and install
cargo install --path .
```

### Pre-built Binaries

Coming soon for Linux, macOS, and Windows.

## Quick Start

```bash
# Initialize EnvelopeCLI
envelope init

# Create your first account
envelope account create "Checking" --balance 100000  # $1000.00 in cents

# View your accounts
envelope account list

# Launch the TUI
envelope tui
```

## Usage

### CLI Commands

```bash
# Account management
envelope account create <name> [--type checking|savings|credit|cash] [--balance <cents>]
envelope account list [--all]
envelope account show <name>
envelope account archive <name>

# Category management
envelope category create <name> --group <group>
envelope category list
envelope category create-group <name>

# Budget operations
envelope budget assign <category> <amount> [--period 2025-01]
envelope budget move <from> <to> <amount>
envelope budget overview [--period 2025-01]

# Transactions
envelope transaction add <account> <amount> [--payee <name>] [--category <name>]
envelope transaction list [--account <name>] [--limit <n>]
envelope transaction import <file.csv> --account <name>

# TUI mode
envelope tui
```

### TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` | Quit |
| `?` | Help |
| `j/k` | Navigate down/up |
| `h/l` | Navigate left/right (panels) |
| `Enter` | Select |
| `a` | Add transaction |
| `e` | Edit selected |
| `d` | Delete selected |
| `c` | Clear/mark transaction |
| `m` | Move funds |
| `:` | Command palette |

## Data Storage

EnvelopeCLI stores all data locally in `~/.envelope/`:

```
~/.envelope/
├── config.json          # User preferences
├── data/
│   ├── accounts.json    # Account definitions
│   ├── budget.json      # Categories and allocations
│   ├── transactions.json # All transactions
│   └── payees.json      # Payee rules
├── audit.log            # Change history
└── backups/             # Automatic backups
```

## Development

### Requirements

- Rust 1.70+
- Cargo

### Building

```bash
cargo build
cargo test
cargo run -- --help
```

### Project Structure

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library root
├── config/          # Configuration management
├── models/          # Data models
├── storage/         # JSON storage layer
├── services/        # Business logic
├── cli/             # CLI command handlers
├── tui/             # Terminal UI
├── audit/           # Audit logging
├── backup/          # Backup management
└── error.rs         # Error types
```

## License

MIT License - See LICENSE for details.

## Author

Kaylee Beyene ([@coderkaylee](https://twitter.com/coderkaylee))
