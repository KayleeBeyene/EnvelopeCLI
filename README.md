# EnvelopeCLI

A terminal-based zero-based budgeting application inspired by YNAB. Every dollar gets a job.

## Features

### Core Budgeting
- **Zero-based budgeting** - Assign every dollar to a category before you spend it
- **Budget targets** - Set recurring targets (weekly, monthly, yearly, or by specific date) with auto-fill
- **Fund movement** - Move money between categories as priorities change
- **Rollover support** - Carry over surplus or deficit from previous periods
- **Overspending alerts** - Track and resolve overspent categories

### Account Management
- **Multiple account types** - Checking, savings, credit cards, cash, investments, lines of credit
- **On/off-budget accounts** - Track investment accounts without affecting your budget
- **Account archiving** - Hide accounts without losing historical data
- **Net worth tracking** - See your complete financial picture

### Transaction Management
- **Full transaction tracking** - Date, payee, category, memo, and cleared status
- **CSV import** - Import transactions from your bank
- **Transfers** - Move money between accounts with linked transactions
- **Bulk operations** - Categorize multiple transactions at once
- **Reconciliation** - Match your records with bank statements

### Reporting
- **Budget overview** - See budgeted, spent, and available by category
- **Spending analysis** - Track spending by category with percentage breakdowns
- **Account register** - Filter transactions by date, payee, or category
- **Net worth report** - Assets vs liabilities summary

### Data & Security
- **Local-first storage** - All data stored on your machine in JSON format
- **AES-256-GCM encryption** - Optional passphrase-protected encryption with Argon2 key derivation
- **Automatic backups** - Configurable backup retention
- **Multi-format export** - CSV, JSON, or YAML for portability

### Interface
- **Interactive TUI** - Full terminal interface with vim-style navigation
- **CLI commands** - Script-friendly command-line interface for automation
- **Command palette** - Quick access to all actions with `:` key

## Installation

### Via Cargo (Recommended)

```bash
cargo install envelope-cli
```

### Via Homebrew (macOS/Linux)

```bash
brew tap KayleeBeyene/tap
brew install envelope-cli
```

> **Note:** Use `envelope-cli` (not `envelope`) to avoid conflict with an unrelated package in homebrew-core.

### Via Shell Script (macOS/Linux)

```bash
curl -fsSL https://github.com/KayleeBeyene/EnvelopeCLI/releases/latest/download/envelope-installer.sh | sh
```

### Via PowerShell (Windows)

```powershell
irm https://github.com/KayleeBeyene/EnvelopeCLI/releases/latest/download/envelope-installer.ps1 | iex
```

### From Source

```bash
git clone https://github.com/KayleeBeyene/EnvelopeCLI.git
cd EnvelopeCLI
cargo install --path .
```

Requires Rust 1.70+

## Quick Start

```bash
# Initialize with default categories
envelope init

# Create your first account
envelope account create "Checking" --balance 2500.00

# Launch the TUI
envelope tui
```

## CLI Reference

### Account Commands

```bash
envelope account create "Savings" --type savings --balance 5000.00
envelope account create "Visa" --type credit --balance -1200.00
envelope account list                    # Show all active accounts
envelope account list --all              # Include archived accounts
envelope account show "Checking"         # View account details
envelope account edit "Checking" --name "Primary Checking"
envelope account archive "Old Account"   # Hide without deleting
envelope account unarchive "Old Account"
```

### Transaction Commands

```bash
envelope transaction add "Checking" -50.00 --payee "Grocery Store" --category "Groceries"
envelope transaction list --account "Checking" --limit 20
envelope txn add "Checking" 2000.00 --payee "Employer" --memo "Paycheck"
```

### Transfer Between Accounts

```bash
envelope transfer "Checking" "Savings" 500.00 --memo "Monthly savings"
```

### Budget Commands

```bash
envelope budget overview                       # Current month overview
envelope budget overview --period 2025-01      # Specific month
envelope budget assign "Groceries" 400.00      # Assign funds to category
envelope budget move "Dining Out" "Groceries" 50.00  # Move funds between categories
envelope budget rollover                       # Apply previous month's rollover
envelope budget overspent                      # List overspent categories
envelope budget periods --count 6              # Show recent budget periods
```

### Target Commands (Recurring Goals)

```bash
envelope target set "Rent" 1500.00 --cadence monthly
envelope target set "Car Insurance" 600.00 --cadence yearly
envelope target set "Vacation" 2000.00 --cadence by-date --date 2025-06-01
envelope target list                           # Show all targets
envelope target show "Rent"                    # View target details
envelope target auto-fill                      # Fill budgets from targets
envelope target delete "Rent"                  # Remove a target
```

### Category Commands

```bash
envelope category create "Coffee" --group "Wants"
envelope category list
envelope category create-group "Side Hustle"
```

### Report Commands

```bash
envelope report budget                         # Budget overview
envelope report spending --period 2025-01      # Spending by category
envelope report spending --top 5               # Top 5 spending categories
envelope report register "Checking"            # Account transaction history
envelope report net-worth                      # Assets vs liabilities
envelope report register "Checking" --output transactions.csv
```

### Export Commands

```bash
envelope export all backup.json --format json --pretty
envelope export all backup.yaml --format yaml
envelope export transactions transactions.csv
envelope export accounts accounts.csv
envelope export allocations budget-history.csv --months 12
envelope export info                           # Show data summary
```

### Import Transactions

```bash
envelope import bank-export.csv --account "Checking"
```

### Encryption Commands

```bash
envelope encrypt enable                        # Enable AES-256-GCM encryption
envelope encrypt status                        # Check encryption status
envelope encrypt verify                        # Verify your passphrase
envelope encrypt change-passphrase             # Change passphrase
envelope encrypt disable                       # Remove encryption
```

### Backup Commands

```bash
envelope backup create                         # Create manual backup
envelope backup list                           # List available backups
envelope backup restore <backup-file>          # Restore from backup
```

### Other Commands

```bash
envelope config                                # Show configuration and paths
envelope init                                  # Initialize new budget
envelope tui                                   # Launch interactive TUI
envelope --help                                # Show all commands
```

## TUI Keyboard Shortcuts

### Global

| Key | Action |
|-----|--------|
| `q` | Quit |
| `?` | Help dialog |
| `:` | Command palette |
| `Tab` | Switch panel focus |
| `h/l` | Focus sidebar/main panel |
| `j/k` | Navigate down/up |
| `1` | Accounts view |
| `2` | Budget view |
| `3` | Reports view |

### Register View (Transactions)

| Key | Action |
|-----|--------|
| `a` | Add transaction |
| `e` | Edit selected transaction |
| `c` | Toggle cleared status |
| `Ctrl+d` | Delete transaction |
| `v` | Multi-select mode |
| `Space` | Toggle selection (in multi-select) |
| `g` | Go to top |
| `G` | Go to bottom |

### Budget View

| Key | Action |
|-----|--------|
| `[` / `]` | Previous/next period |
| `m` | Move funds between categories |
| `a` | Add category |
| `A` | Add category group |
| `Enter` | Edit budget/target for category |

### Sidebar

| Key | Action |
|-----|--------|
| `a` | Add account |
| `Enter` | Select account |
| `A` | Toggle archived accounts |

### Dialogs

| Key | Action |
|-----|--------|
| `Esc` | Close/cancel |
| `Enter` | Confirm |
| `Tab` | Next field |

## Data Storage

All data is stored locally in `~/.envelope/`:

```
~/.envelope/
├── config.json          # Settings and encryption config
├── data/
│   ├── accounts.json    # Account definitions
│   ├── categories.json  # Category groups and categories
│   ├── allocations.json # Budget allocations per period
│   ├── transactions.json
│   ├── payees.json      # Payee rules for auto-categorization
│   └── targets.json     # Recurring budget targets
├── audit.log            # Change history
└── backups/             # Automatic backups
```

## Project Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library root
├── cli/                 # CLI command handlers
├── tui/                 # Terminal UI (ratatui)
│   ├── app.rs           # Application state
│   ├── views/           # Main views (accounts, register, budget)
│   ├── dialogs/         # Modal dialogs
│   └── widgets/         # Reusable UI components
├── models/              # Data models (Account, Transaction, Category, etc.)
├── services/            # Business logic layer
├── storage/             # JSON persistence layer
├── reports/             # Report generation
├── export/              # CSV, JSON, YAML export
├── crypto/              # Encryption (AES-256-GCM, Argon2)
├── backup/              # Backup management
├── audit/               # Audit logging
└── error.rs             # Error types
```

## Development

```bash
cargo build              # Build
cargo test               # Run tests
cargo run -- --help      # Run with args
cargo clippy             # Lint
cargo fmt                # Format
```

## License

MIT License - See LICENSE for details.

## Author

Kaylee Beyene ([@coderkaylee](https://twitter.com/coderkaylee))
