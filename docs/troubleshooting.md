# EnvelopeCLI Troubleshooting Guide

This guide helps you diagnose and resolve common issues with EnvelopeCLI.

## Quick Diagnostics

Run these commands to gather diagnostic information:

```bash
# Show version and configuration
envelope --version
envelope config

# Check if data files exist
ls -la ~/.envelope/
ls -la ~/.envelope/data/
```

---

## Installation Issues

### Binary Not Found

**Symptom:** `command not found: envelope`

**Solutions:**

1. Ensure the binary is in your PATH:
   ```bash
   # Check where envelope is installed
   which envelope

   # Add to PATH if needed (add to ~/.bashrc or ~/.zshrc)
   export PATH="$PATH:/path/to/envelope"
   ```

2. On macOS, you may need to allow the binary:
   ```bash
   # If you see "cannot be opened" error
   xattr -d com.apple.quarantine /path/to/envelope
   ```

3. On Linux, ensure the binary is executable:
   ```bash
   chmod +x /path/to/envelope
   ```

### Permission Denied

**Symptom:** `Permission denied` when running envelope

**Solutions:**

```bash
# Make the binary executable
chmod +x /usr/local/bin/envelope

# Or move to a user-writable location
mv envelope ~/bin/
```

---

## Data Issues

### Data Directory Not Found

**Symptom:** `Configuration error: Failed to create directory`

**Solutions:**

1. Create the directory manually:
   ```bash
   mkdir -p ~/.envelope/data
   mkdir -p ~/.envelope/backups
   ```

2. Check permissions:
   ```bash
   ls -la ~/
   # Ensure you own your home directory
   ```

### Corrupted Data File

**Symptom:** `JSON error: Failed to parse` or `Data file is corrupted`

**Solutions:**

1. Restore from backup:
   ```bash
   # List available backups
   envelope backup list

   # Restore most recent
   envelope backup restore ~/.envelope/backups/YYYY-MM-DD_HHMMSS.json
   ```

2. Manually inspect the file:
   ```bash
   # Check if it's valid JSON
   cat ~/.envelope/data/transactions.json | python3 -m json.tool

   # If invalid, check the audit log for recent changes
   tail -50 ~/.envelope/audit.log
   ```

3. If no backup exists, check for `.tmp` files that might contain good data:
   ```bash
   ls -la ~/.envelope/data/*.tmp
   ```

### Audit Log Too Large

**Symptom:** Slow startup, large audit.log file

**Solution:** The audit log can be safely truncated (it's informational):

```bash
# Backup the old log first
mv ~/.envelope/audit.log ~/.envelope/audit.log.old

# Create a fresh log
touch ~/.envelope/audit.log
```

---

## TUI Issues

### Terminal Display Problems

**Symptom:** Garbled display, wrong colors, or layout issues

**Solutions:**

1. Ensure your terminal supports colors:
   ```bash
   echo $TERM
   # Should be xterm-256color, screen-256color, or similar
   ```

2. Try a different terminal emulator (iTerm2, Alacritty, Kitty work well)

3. Reset terminal state:
   ```bash
   reset
   ```

4. Check terminal size (minimum 80x24 recommended):
   ```bash
   echo "Columns: $(tput cols), Lines: $(tput lines)"
   ```

### Keyboard Not Working

**Symptom:** Keys don't respond in TUI

**Solutions:**

1. Ensure you're not in tmux with conflicting bindings
2. Try running outside of screen/tmux first
3. Check if you have a keyboard mapping issue:
   ```bash
   # Test raw key input
   cat -v
   # Press keys to see what's being sent
   ```

### TUI Won't Start

**Symptom:** `TUI error: Failed to initialize terminal`

**Solutions:**

1. Ensure you're running in a real terminal (not piped):
   ```bash
   # This won't work
   echo "test" | envelope tui

   # Run directly
   envelope tui
   ```

2. Try the CLI instead:
   ```bash
   envelope budget status
   envelope transaction list Checking
   ```

---

## Budget Issues

### Available to Budget Shows Wrong Amount

**Symptom:** ATB doesn't match expected amount

**Diagnosis:**

1. Check income transactions (should be uncategorized or to "Income"):
   ```bash
   envelope transaction list Checking --from 2025-01-01
   ```

2. Verify all allocations:
   ```bash
   envelope budget overview
   ```

3. Check for off-budget accounts (they don't contribute to ATB):
   ```bash
   envelope account list
   ```

### Category Balance Incorrect

**Symptom:** Category shows wrong available amount

**Solutions:**

1. Check for uncleared transactions:
   ```bash
   envelope transaction list <account> | grep Pending
   ```

2. Verify the category for transactions:
   ```bash
   envelope report spending --category "Category Name"
   ```

3. Check for split transactions that might be miscategorized

---

## Import Issues

### CSV Import Fails

**Symptom:** `Import error: Failed to parse CSV`

**Solutions:**

1. Check CSV format:
   - Ensure it has a header row
   - Check for consistent column count
   - Look for special characters or encoding issues

2. Try a different preset:
   ```bash
   envelope import file.csv --account Checking --preset chase
   envelope import file.csv --account Checking --preset bofa
   ```

3. Check the date format in your CSV (should be recognizable)

### Duplicate Transactions

**Symptom:** Same transaction imported multiple times

**Solutions:**

1. Use the skip-duplicates flag:
   ```bash
   envelope import file.csv --account Checking --skip-duplicates
   ```

2. Manually delete duplicates:
   ```bash
   envelope transaction list Checking | grep "duplicate payee"
   envelope transaction delete <ID>
   ```

---

## Encryption Issues

### Wrong Passphrase

**Symptom:** `Encryption error: Decryption failed`

**Important:** There is no password recovery. If you've forgotten your passphrase, encrypted data cannot be recovered.

**If you remember a similar passphrase:**
- Try common variations (caps lock, common substitutions)
- Try recently used passwords

### Cannot Disable Encryption

**Symptom:** Want to disable encryption but it requires the passphrase

**Solution:** You must know the passphrase to disable encryption. This is a security feature.

If you truly cannot remember:
1. Your data is not recoverable
2. Delete the data directory and start fresh:
   ```bash
   rm -rf ~/.envelope/
   envelope init
   ```

---

## Performance Issues

### Slow Startup

**Symptom:** Application takes long to start

**Solutions:**

1. Check data file sizes:
   ```bash
   ls -lh ~/.envelope/data/
   ```

2. If transactions.json is very large (>10MB), consider:
   - Archiving old accounts
   - Exporting old data and starting fresh

3. Check for disk issues:
   ```bash
   df -h ~/.envelope/
   ```

### High Memory Usage

**Symptom:** Application uses too much RAM

**Solutions:**

1. Close the TUI and use CLI commands instead
2. If you have many transactions (>50,000), consider archiving old data

---

## Getting Help

### Reporting Bugs

When reporting issues, please include:

1. EnvelopeCLI version: `envelope --version`
2. Operating system and version
3. Terminal emulator name and version
4. Steps to reproduce the issue
5. Any error messages (full text)
6. Relevant log entries from `~/.envelope/audit.log`

### Debug Mode

For more verbose output, set the environment variable:

```bash
RUST_LOG=debug envelope <command>
```

### Reset Everything

If all else fails, you can reset to a fresh state:

```bash
# BACKUP FIRST!
cp -r ~/.envelope/ ~/.envelope.backup/

# Reset
rm -rf ~/.envelope/
envelope init
```

---

## Common Error Messages

| Error | Meaning | Solution |
|-------|---------|----------|
| `Account not found` | Invalid account name/ID | Check `envelope account list` |
| `Category not found` | Invalid category name/ID | Check `envelope category list` |
| `Transaction is locked` | Trying to edit reconciled txn | Unlock first or undo reconciliation |
| `Insufficient funds` | Category doesn't have enough | Move funds or adjust allocation |
| `Invalid amount` | Couldn't parse money value | Use format like "100.00" |
| `File not found` | Missing data file | Run `envelope init` |
| `Permission denied` | Can't write to data dir | Check file permissions |
