# Releasing EnvelopeCLI

This document describes the complete release process for EnvelopeCLI.

## Prerequisites

- [ ] `cargo` and `gh` CLI tools installed
- [ ] Authenticated with crates.io (`cargo login`)
- [ ] Authenticated with GitHub (`gh auth login`)
- [ ] `HOMEBREW_TAP_TOKEN` secret configured in GitHub repo settings

## Release Checklist

### 1. Prepare the Release

```bash
# Ensure you're on master and up to date
git checkout master
git pull origin master

# Run tests locally
cargo test
cargo clippy
cargo fmt --check
```

### 2. Update Version Number

Edit `Cargo.toml` and update the version:

```toml
[package]
name = "envelope-cli"
version = "X.Y.Z"  # Update this
```

> **Note:** The TUI sidebar version display uses `env!("CARGO_PKG_VERSION")` at compile time, so updating `Cargo.toml` automatically updates the version shown in the UI. No manual changes needed elsewhere.

Commit the version bump:

```bash
git add Cargo.toml Cargo.lock
git commit -m "Bump version to X.Y.Z"
git push origin master
```

### 3. Publish to crates.io

```bash
# Dry run first
cargo publish --dry-run

# If successful, publish
cargo publish
```

**Verify:** https://crates.io/crates/envelope-cli

### 4. Create Git Tag and Push

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

This triggers the GitHub Actions release workflow which:
- Builds binaries for macOS (Intel + ARM), Linux, Windows
- Creates GitHub Release with all artifacts
- Generates shell/PowerShell installers
- Generates Homebrew formula (attached to release)

### 5. Monitor Release Workflow

```bash
# Watch the release workflow
gh run list --workflow=release.yml --limit 1
gh run watch <run-id>
```

**Verify:** https://github.com/KayleeBeyene/EnvelopeCLI/releases

### 6. Update Homebrew Tap

The release workflow generates `envelope-cli.rb` but doesn't auto-push to the tap.
You must manually update the tap:

```bash
# Download the formula from the release
gh release download vX.Y.Z --pattern "envelope-cli.rb" --dir /tmp

# Clone the tap repo
gh repo clone KayleeBeyene/homebrew-tap /tmp/homebrew-tap

# Copy the formula (keep the name as envelope-cli.rb)
cp /tmp/envelope-cli.rb /tmp/homebrew-tap/Formula/envelope-cli.rb

# Commit and push
git -C /tmp/homebrew-tap add -A
git -C /tmp/homebrew-tap commit -m "Update envelope-cli formula to vX.Y.Z"
git -C /tmp/homebrew-tap push origin main

# Cleanup
rm -rf /tmp/homebrew-tap /tmp/envelope-cli.rb
```

**Verify:** https://github.com/KayleeBeyene/homebrew-tap

## Distribution Channels

After release, users can install via:

| Method | Command |
|--------|---------|
| Cargo | `cargo install envelope-cli` |
| Homebrew | `brew tap KayleeBeyene/tap && brew install envelope-cli` |
| Shell script | `curl -fsSL https://github.com/KayleeBeyene/EnvelopeCLI/releases/latest/download/envelope-cli-installer.sh \| sh` |
| PowerShell | `irm https://github.com/KayleeBeyene/EnvelopeCLI/releases/latest/download/envelope-cli-installer.ps1 \| iex` |
| Source | `git clone ... && cargo install --path .` |

## Troubleshooting

### crates.io publish fails

- **"no token found"**: Run `cargo login` with your API token from https://crates.io/settings/tokens
- **"email not verified"**: Verify email at https://crates.io/settings/profile
- **"crate exists but you're not owner"**: The crate name is taken; you can't publish to it

### GitHub Actions release fails

- **Check workflow logs**: `gh run view <run-id> --log-failed`
- **Re-run failed workflow**: `gh run rerun <run-id>`

### Homebrew tap not updating

- Ensure `HOMEBREW_TAP_TOKEN` secret exists with write access to `homebrew-tap` repo
- Manually push formula as described in step 6

## Version History

| Version | Date | Notes |
|---------|------|-------|
| 0.2.4 | 2025-11-29 | Expected income tracking with budget warnings |
| 0.2.3 | 2025-11-29 | Vim keybindings, unified backup/export restore |
| 0.2.2 | 2025-11-29 | Patch release |
| 0.2.1 | 2025-11-29 | Category group editing, bulk delete, header-less CSV |
| 0.2.0 | 2025-11-29 | XDG-compliant data paths, env var override |
| 0.1.0 | 2025-11-28 | Initial public release |
