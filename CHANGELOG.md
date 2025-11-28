# Changelog

All notable changes to EnvelopeCLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-11-28

### Added

- **Zero-based budgeting** - Assign every dollar to a category before spending
- **Budget targets** - Set recurring targets (weekly, monthly, yearly, by-date) with auto-fill
- **Interactive TUI** - Full terminal interface with vim-style navigation
- **CLI commands** - Script-friendly command-line interface for automation
- **Multiple account types** - Checking, savings, credit cards, cash, investments, lines of credit
- **On/off-budget accounts** - Track investments without affecting your budget
- **Full transaction tracking** - Date, payee, category, memo, and cleared status
- **CSV import** - Import transactions from bank exports
- **Transfers** - Move money between accounts with linked transactions
- **Bulk operations** - Categorize multiple transactions at once
- **Account reconciliation** - Match records with bank statements
- **Budget overview reports** - See budgeted, spent, and available by category
- **Spending analysis** - Track spending by category with percentage breakdowns
- **Net worth report** - Assets vs liabilities summary
- **AES-256-GCM encryption** - Optional passphrase-protected encryption with Argon2 key derivation
- **Automatic backups** - Configurable backup retention
- **Multi-format export** - CSV, JSON, or YAML for portability
- **Command palette** - Quick access to all TUI actions with `:` key
- **Setup wizard** - Interactive first-run configuration
- **Audit logging** - Track all data changes

### Technical

- Local-first JSON storage with atomic writes
- Cross-platform support (Linux, macOS, Windows)
- Rust 1.70+ required

[Unreleased]: https://github.com/KayleeBeyene/EnvelopeCLI/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/KayleeBeyene/EnvelopeCLI/releases/tag/v0.1.0
