# EnvelopeCLI Command Reference

This document provides a complete reference for all CLI commands available in EnvelopeCLI.

## Global Options

| Option | Description |
|--------|-------------|
| `--help`, `-h` | Show help message |
| `--version`, `-V` | Show version information |

## Commands Overview

| Command | Description |
|---------|-------------|
| `tui` | Launch the interactive TUI |
| `init` | Initialize a new budget |
| `config` | Show current configuration |
| `account` | Account management |
| `category` | Category management |
| `budget` | Budget allocation |
| `transaction` | Transaction management |
| `transfer` | Account transfers |
| `payee` | Payee management |
| `reconcile` | Account reconciliation |
| `import` | Import transactions from CSV |
| `export` | Export data |
| `report` | Generate reports |
| `backup` | Backup management |
| `encrypt` | Encryption management |

---

## Account Commands

### `envelope account create`

Create a new account.

```bash
envelope account create <NAME> --type <TYPE> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Account name (e.g., "Checking", "Savings")

**Options:**
- `--type`, `-t` - Account type: `checking`, `savings`, `credit`, `cash`, `investment`, `other`
- `--off-budget` - Mark as off-budget (doesn't affect Available to Budget)
- `--balance`, `-b` - Starting balance (e.g., "1000.00")

**Examples:**
```bash
# Create a checking account
envelope account create "Chase Checking" --type checking

# Create a savings account with starting balance
envelope account create "Emergency Fund" --type savings --balance 5000.00

# Create an off-budget investment account
envelope account create "401k" --type investment --off-budget
```

### `envelope account list`

List all accounts with balances.

```bash
envelope account list [OPTIONS]
```

**Options:**
- `--archived` - Include archived accounts
- `--format` - Output format: `table` (default), `json`

### `envelope account show`

Show details for a specific account.

```bash
envelope account show <NAME_OR_ID>
```

### `envelope account edit`

Edit an existing account.

```bash
envelope account edit <NAME_OR_ID> [OPTIONS]
```

**Options:**
- `--name`, `-n` - New name
- `--type`, `-t` - New type
- `--on-budget/--off-budget` - Change budget status

### `envelope account archive`

Archive (soft-delete) an account.

```bash
envelope account archive <NAME_OR_ID>
```

---

## Category Commands

### `envelope category create`

Create a new category.

```bash
envelope category create <NAME> --group <GROUP> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Category name

**Options:**
- `--group`, `-g` - Category group (required)

**Examples:**
```bash
envelope category create "Coffee" --group "Wants"
envelope category create "Car Payment" --group "Bills"
```

### `envelope category list`

List all categories organized by group.

```bash
envelope category list [OPTIONS]
```

**Options:**
- `--format` - Output format: `tree` (default), `flat`, `json`

### `envelope category delete`

Delete a category.

```bash
envelope category delete <NAME_OR_ID>
```

### `envelope category move`

Move a category to a different group.

```bash
envelope category move <NAME_OR_ID> --to <GROUP>
```

---

## Budget Commands

### `envelope budget assign`

Assign funds to a category.

```bash
envelope budget assign <CATEGORY> <AMOUNT> [OPTIONS]
```

**Arguments:**
- `<CATEGORY>` - Category name or ID
- `<AMOUNT>` - Amount to assign (e.g., "500.00")

**Options:**
- `--period`, `-p` - Budget period (defaults to current month)

**Examples:**
```bash
# Assign $500 to Groceries for current month
envelope budget assign Groceries 500.00

# Assign for a specific month
envelope budget assign "Rent" 1500.00 --period 2025-02
```

### `envelope budget move`

Move funds between categories.

```bash
envelope budget move <FROM> <TO> <AMOUNT> [OPTIONS]
```

**Arguments:**
- `<FROM>` - Source category
- `<TO>` - Destination category
- `<AMOUNT>` - Amount to move

### `envelope budget status`

Show current budget status.

```bash
envelope budget status [OPTIONS]
```

**Options:**
- `--period`, `-p` - Budget period (defaults to current)

### `envelope budget overview`

Show full budget overview with all categories.

```bash
envelope budget overview [OPTIONS]
```

**Options:**
- `--period`, `-p` - Budget period

---

## Transaction Commands

### `envelope transaction add`

Add a new transaction.

```bash
envelope transaction add <ACCOUNT> <AMOUNT> [OPTIONS]
```

**Arguments:**
- `<ACCOUNT>` - Account name or ID
- `<AMOUNT>` - Amount (negative for outflow, positive for inflow)

**Options:**
- `--payee`, `-p` - Payee name
- `--category`, `-c` - Category name or ID
- `--date`, `-d` - Date (YYYY-MM-DD, defaults to today)
- `--memo`, `-m` - Memo/notes
- `--cleared` - Mark as cleared
- `--split` - Add split (can be repeated): `--split Category:Amount`

**Examples:**
```bash
# Simple expense
envelope transaction add Checking -50.00 --payee "Grocery Store" --category Groceries

# Income
envelope transaction add Checking 3000.00 --payee "Employer" --category "Income" --cleared

# Split transaction
envelope transaction add Checking -100.00 --payee "Target" \
  --split "Groceries:60.00" \
  --split "Household:40.00"
```

### `envelope transaction list`

List transactions for an account.

```bash
envelope transaction list <ACCOUNT> [OPTIONS]
```

**Options:**
- `--from` - Start date (YYYY-MM-DD)
- `--to` - End date (YYYY-MM-DD)
- `--limit`, `-n` - Number of transactions to show
- `--format` - Output format: `table` (default), `json`

### `envelope transaction edit`

Edit an existing transaction.

```bash
envelope transaction edit <ID> [OPTIONS]
```

**Options:**
- Same as `add` command

### `envelope transaction delete`

Delete a transaction.

```bash
envelope transaction delete <ID>
```

### `envelope transaction clear`

Mark a transaction as cleared.

```bash
envelope transaction clear <ID>
```

---

## Transfer Command

Transfer funds between accounts.

```bash
envelope transfer <FROM_ACCOUNT> <TO_ACCOUNT> <AMOUNT> [OPTIONS]
```

**Options:**
- `--date`, `-d` - Transfer date
- `--memo`, `-m` - Memo

**Example:**
```bash
envelope transfer Checking Savings 500.00 --memo "Monthly savings"
```

---

## Import Command

Import transactions from a CSV file.

```bash
envelope import <FILE> --account <ACCOUNT> [OPTIONS]
```

**Options:**
- `--account`, `-a` - Target account (required)
- `--preset` - Use a column mapping preset (chase, bofa, etc.)
- `--skip-duplicates` - Automatically skip duplicate transactions

**Example:**
```bash
envelope import bank_statement.csv --account Checking --preset chase
```

---

## Export Commands

### `envelope export csv`

Export transactions to CSV.

```bash
envelope export csv [OPTIONS]
```

**Options:**
- `--output`, `-o` - Output file path (default: stdout)
- `--account`, `-a` - Filter by account
- `--from` - Start date
- `--to` - End date

### `envelope export json`

Export all data to JSON.

```bash
envelope export json --output <FILE>
```

### `envelope export yaml`

Export all data to YAML.

```bash
envelope export yaml --output <FILE>
```

---

## Report Commands

### `envelope report budget`

Generate budget overview report.

```bash
envelope report budget [OPTIONS]
```

**Options:**
- `--period`, `-p` - Budget period
- `--csv` - Output as CSV

### `envelope report spending`

Generate spending by category report.

```bash
envelope report spending [OPTIONS]
```

**Options:**
- `--from` - Start date
- `--to` - End date
- `--csv` - Output as CSV

### `envelope report networth`

Show net worth (sum of all accounts).

```bash
envelope report networth [OPTIONS]
```

---

## Reconcile Commands

### `envelope reconcile start`

Start reconciliation for an account.

```bash
envelope reconcile start <ACCOUNT> --balance <BALANCE> [OPTIONS]
```

**Options:**
- `--balance`, `-b` - Statement ending balance (required)
- `--date`, `-d` - Statement date

### `envelope reconcile status`

Show current reconciliation status.

```bash
envelope reconcile status <ACCOUNT>
```

---

## Backup Commands

### `envelope backup create`

Create a backup of all data.

```bash
envelope backup create
```

### `envelope backup list`

List available backups.

```bash
envelope backup list
```

### `envelope backup restore`

Restore from a backup.

```bash
envelope backup restore <BACKUP_FILE>
```

---

## Encrypt Commands

### `envelope encrypt enable`

Enable encryption for your data.

```bash
envelope encrypt enable
```

You will be prompted to enter and confirm a passphrase.

### `envelope encrypt disable`

Disable encryption.

```bash
envelope encrypt disable
```

Requires current passphrase.

### `envelope encrypt change-passphrase`

Change encryption passphrase.

```bash
envelope encrypt change-passphrase
```

### `envelope encrypt status`

Show encryption status.

```bash
envelope encrypt status
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Configuration error |
| 2 | I/O error |
| 3 | Data file error |
| 4 | Validation error |
| 5 | Not found |
| 6 | Duplicate entry |
| 7 | Budget error |
| 8 | Reconciliation error |
| 9 | Import error |
| 10 | Export error |
| 11 | Encryption error |
| 12 | Locked transaction |
| 13 | Insufficient funds |
| 14 | Storage error |
| 15 | TUI error |
