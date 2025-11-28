# Contributing to EnvelopeCLI

Thank you for your interest in contributing to EnvelopeCLI! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates. When creating a bug report, include:

- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior vs actual behavior
- Your environment (OS, Rust version, EnvelopeCLI version)
- Any relevant error messages or logs

### Suggesting Features

Feature suggestions are welcome! Please:

- Check existing issues and discussions first
- Describe the problem your feature would solve
- Explain your proposed solution
- Consider how it fits with the project's goals

### Pull Requests

1. **Fork and clone** the repository
2. **Create a branch** for your changes: `git checkout -b feature/your-feature-name`
3. **Make your changes** following our coding standards
4. **Write or update tests** as needed
5. **Run the test suite**: `cargo test`
6. **Run lints**: `cargo clippy`
7. **Format code**: `cargo fmt`
8. **Commit your changes** with a descriptive message
9. **Push** to your fork and create a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git

### Building

```bash
git clone https://github.com/KayleeBeyene/EnvelopeCLI.git
cd EnvelopeCLI
cargo build
```

### Running Tests

```bash
cargo test          # Run all tests
cargo test -- --nocapture  # See test output
```

### Code Quality

```bash
cargo clippy        # Run lints
cargo fmt           # Format code
cargo doc --open    # Generate and view documentation
```

## Coding Standards

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for consistent formatting
- Address all `clippy` warnings
- Write documentation for public APIs

### Commit Messages

Use clear, descriptive commit messages:

- Start with a verb (Add, Fix, Update, Remove, Refactor)
- Keep the first line under 72 characters
- Reference issues when applicable: `Fix #123`

Examples:
```
Add budget rollover command
Fix category deletion when transactions exist
Update TUI keyboard shortcuts documentation
```

### Testing

- Write unit tests for new functionality
- Ensure existing tests pass
- Test edge cases and error conditions
- TUI code may be excluded from coverage requirements

## Project Structure

```
src/
├── cli/        # CLI command handlers
├── tui/        # Terminal UI (ratatui)
├── models/     # Data models
├── services/   # Business logic
├── storage/    # JSON persistence
├── reports/    # Report generation
├── export/     # CSV, JSON, YAML export
├── crypto/     # Encryption
├── backup/     # Backup management
└── audit/      # Audit logging
```

## Areas for Contribution

- Bug fixes
- Documentation improvements
- Test coverage
- Performance optimizations
- Accessibility improvements
- New export formats
- Import from other budgeting tools

## Questions?

Feel free to open an issue for any questions about contributing.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
