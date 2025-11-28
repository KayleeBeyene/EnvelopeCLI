# Installing EnvelopeCLI

EnvelopeCLI is a terminal-based zero-based budgeting application. This guide covers installation on all supported platforms.

## Quick Install

### From Pre-built Binaries

Download the appropriate binary for your system from the [Releases](https://github.com/KayleeBeyene/EnvelopeCLI/releases) page.

**Linux (x64):**
```bash
curl -L https://github.com/KayleeBeyene/EnvelopeCLI/releases/latest/download/envelope-linux-x64 -o envelope
chmod +x envelope
sudo mv envelope /usr/local/bin/
```

**Linux (ARM64):**
```bash
curl -L https://github.com/KayleeBeyene/EnvelopeCLI/releases/latest/download/envelope-linux-arm64 -o envelope
chmod +x envelope
sudo mv envelope /usr/local/bin/
```

**macOS (Intel):**
```bash
curl -L https://github.com/KayleeBeyene/EnvelopeCLI/releases/latest/download/envelope-macos-x64 -o envelope
chmod +x envelope
sudo mv envelope /usr/local/bin/
```

**macOS (Apple Silicon):**
```bash
curl -L https://github.com/KayleeBeyene/EnvelopeCLI/releases/latest/download/envelope-macos-arm64 -o envelope
chmod +x envelope
sudo mv envelope /usr/local/bin/
```

**Windows:**
Download `envelope-windows-x64.exe` and add it to your PATH.

### From Source

Requirements:
- Rust 1.70 or later
- Git

```bash
# Clone the repository
git clone https://github.com/KayleeBeyene/EnvelopeCLI.git
cd EnvelopeCLI

# Build in release mode
cargo build --release

# Install to ~/.cargo/bin/
cargo install --path .

# Or copy manually
cp target/release/envelope /usr/local/bin/
```

## Platform-Specific Notes

### Linux

**Dependencies:** None required. The binary is statically linked.

**Terminal:** Works with most terminal emulators. Recommended:
- Alacritty
- Kitty
- GNOME Terminal
- Konsole

**Shell Integration:** Add to your shell config:
```bash
# ~/.bashrc or ~/.zshrc
export PATH="$PATH:/usr/local/bin"

# Optional: alias
alias env='envelope'
```

### macOS

**First Run:** macOS may block the binary. Allow it with:
```bash
xattr -d com.apple.quarantine /usr/local/bin/envelope
```

Or go to System Preferences > Security & Privacy and click "Allow Anyway".

**Terminal:** Works with Terminal.app and iTerm2. iTerm2 is recommended for better color support.

**Homebrew (Coming Soon):**
```bash
# Not yet available, but planned:
# brew install envelope-cli
```

### Windows

**Requirements:**
- Windows 10 or later
- Windows Terminal (recommended) or PowerShell

**Installation:**

1. Download `envelope-windows-x64.exe`
2. Rename to `envelope.exe`
3. Move to a directory in your PATH, or add a new directory:

```powershell
# Create a bin directory
mkdir $env:USERPROFILE\bin

# Add to PATH (run as Administrator)
[Environment]::SetEnvironmentVariable(
    "Path",
    $env:Path + ";$env:USERPROFILE\bin",
    "User"
)

# Move the binary
mv envelope.exe $env:USERPROFILE\bin\
```

**Windows Terminal Setup:**
For best experience, use Windows Terminal with a modern font that supports Unicode.

### WSL (Windows Subsystem for Linux)

Use the Linux installation instructions. Works great with WSL2.

```bash
# In WSL terminal
curl -L https://github.com/KayleeBeyene/EnvelopeCLI/releases/latest/download/envelope-linux-x64 -o envelope
chmod +x envelope
sudo mv envelope /usr/local/bin/
```

## Verifying Installation

```bash
# Check version
envelope --version

# Show help
envelope --help

# Show configuration
envelope config
```

## First Run

After installation, initialize your budget:

```bash
# Initialize with setup wizard
envelope init

# Or launch TUI directly (will prompt for setup)
envelope tui
```

## Data Location

EnvelopeCLI stores data in:

| Platform | Location |
|----------|----------|
| Linux | `~/.envelope/` |
| macOS | `~/.envelope/` |
| Windows | `%APPDATA%\envelope\` |

## Updating

### Binary Update

1. Download the new binary
2. Replace the existing binary:
   ```bash
   sudo mv envelope /usr/local/bin/envelope
   ```

### From Source Update

```bash
cd EnvelopeCLI
git pull
cargo build --release
cargo install --path . --force
```

## Uninstalling

```bash
# Remove binary
sudo rm /usr/local/bin/envelope

# Optional: Remove data (WARNING: This deletes all your budget data!)
rm -rf ~/.envelope/
```

On Windows:
```powershell
del $env:USERPROFILE\bin\envelope.exe

# Optional: Remove data
Remove-Item -Recurse $env:APPDATA\envelope
```

## Troubleshooting

See [docs/troubleshooting.md](docs/troubleshooting.md) for common issues.

### Common Issues

**"command not found"**
- Ensure the binary is in your PATH
- Restart your terminal after installation

**"Permission denied"**
- Make the binary executable: `chmod +x envelope`
- On macOS, remove quarantine: `xattr -d com.apple.quarantine envelope`

**Build fails**
- Ensure Rust is up to date: `rustup update stable`
- Check for missing dependencies on Linux

## Getting Help

- [Documentation](docs/)
- [Issues](https://github.com/KayleeBeyene/EnvelopeCLI/issues)
- [Discussions](https://github.com/KayleeBeyene/EnvelopeCLI/discussions)
