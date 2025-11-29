# Changelog

All notable changes to EnvelopeCLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3] - 2025-11-29

### Added

- **TUI vim keybindings** - Navigate with `h/j/k/l` keys in addition to arrow keys
- **Unified backup/export restore** - `backup restore` now auto-detects and restores both internal backup format and export format files (YAML/JSON)

### Fixed

- CSV header detection now handles edge cases more reliably

## [0.2.2] - 2025-11-29

### Fixed

- Minor patch release

## [0.2.1] - 2025-11-29

### Added

- **Category group management** - Edit and delete category groups from the TUI
- **Bulk transaction delete** - Delete multiple transactions at once
- **Header-less CSV import** - Import CSV files without headers by specifying column order
- **Dynamic version display** - Sidebar now shows actual package version

### Fixed

- Removed unnecessary format! macro in budget view

## [0.2.0] - 2025-11-29

### Changed

- **Data location** - Moved from platform-specific paths to `~/.config/envelope-cli/` on Unix and `%APPDATA%\envelope-cli\` on Windows
- **XDG compliance** - Respects `XDG_CONFIG_HOME` environment variable when set
- **Environment override** - Added `ENVELOPE_CLI_DATA_DIR` for custom data location
- **Naming** - Directory renamed from `envelope` to `envelope-cli` to avoid conflicts with other packages

### Removed

- Removed `directories` crate dependency in favor of direct XDG path resolution

### Migration

If upgrading from 0.1.x, move your data manually:

```bash
# macOS (from Application Support)
mv ~/Library/Application\ Support/envelope ~/.config/envelope-cli

# Linux (if using old path)
mv ~/.envelope ~/.config/envelope-cli
```

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

[Unreleased]: https://github.com/KayleeBeyene/EnvelopeCLI/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/KayleeBeyene/EnvelopeCLI/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/KayleeBeyene/EnvelopeCLI/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/KayleeBeyene/EnvelopeCLI/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/KayleeBeyene/EnvelopeCLI/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/KayleeBeyene/EnvelopeCLI/releases/tag/v0.1.0
