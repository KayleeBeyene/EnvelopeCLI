# EnvelopeCLI Keyboard Shortcuts

This document lists all keyboard shortcuts available in the TUI (Terminal User Interface).

## Global Shortcuts

These shortcuts work in any view.

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `?` | Show help overlay |
| `:` or `/` | Open command palette |
| `Tab` | Switch between panels |
| `Esc` | Close dialog/cancel |
| `h`, `Left` | Navigate left/previous panel |
| `l`, `Right` | Navigate right/next panel |

## Navigation

| Key | Action |
|-----|--------|
| `j`, `Down` | Move down in list |
| `k`, `Up` | Move up in list |
| `g`, `Home` | Go to first item |
| `G`, `End` | Go to last item |
| `Page Up` | Scroll up one page |
| `Page Down` | Scroll down one page |
| `Enter` | Select/open item |

## Account List (Sidebar)

| Key | Action |
|-----|--------|
| `j`/`k` | Navigate accounts |
| `Enter` | View account transactions |
| `a` | Add new account |
| `e` | Edit selected account |
| `A` | Archive selected account |

## Transaction Register

| Key | Action |
|-----|--------|
| `a` | Add new transaction |
| `e` | Edit selected transaction |
| `d` | Delete selected transaction |
| `c` | Toggle cleared status |
| `Space` | Select/deselect for bulk operations |
| `C` | Clear all selected transactions |
| `B` | Bulk categorize selected |
| `r` | Start reconciliation |

## Budget View

| Key | Action |
|-----|--------|
| `j`/`k` | Navigate categories |
| `Enter` | Edit budget amount |
| `m` | Move funds between categories |
| `[` | Previous period |
| `]` | Next period |
| `t` | Go to current period (today) |

## Reconciliation View

| Key | Action |
|-----|--------|
| `j`/`k` | Navigate transactions |
| `c` | Toggle cleared status |
| `Space` | Mark as cleared |
| `Enter` | Complete reconciliation (when balanced) |
| `Esc` | Cancel reconciliation |

## Dialogs

### Transaction Entry Dialog

| Key | Action |
|-----|--------|
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Enter` | Save transaction |
| `Esc` | Cancel |
| `Ctrl+S` | Save and add another |

### Command Palette

| Key | Action |
|-----|--------|
| Type | Filter commands |
| `j`/`k`, `Up`/`Down` | Navigate commands |
| `Enter` | Execute command |
| `Esc` | Close palette |

### Help Overlay

| Key | Action |
|-----|--------|
| `?`, `Esc`, `Enter` | Close help |
| `j`/`k` | Scroll help text |

## Text Input

| Key | Action |
|-----|--------|
| Type | Insert characters |
| `Backspace` | Delete previous character |
| `Delete` | Delete next character |
| `Ctrl+A` | Select all |
| `Ctrl+U` | Clear input |
| `Left`/`Right` | Move cursor |
| `Home` | Go to beginning |
| `End` | Go to end |

## Multi-Select Mode

When in multi-select mode (after pressing `Space` in transaction register):

| Key | Action |
|-----|--------|
| `Space` | Toggle selection |
| `a` | Select all |
| `n` | Deselect all |
| `Esc` | Exit multi-select |

## Vim-style Navigation

For users familiar with Vim, EnvelopeCLI supports vim-style navigation:

| Key | Vim Equivalent |
|-----|----------------|
| `h` | Left |
| `j` | Down |
| `k` | Up |
| `l` | Right |
| `g` | Go to top |
| `G` | Go to bottom |
| `w` | Next word (in text fields) |
| `b` | Previous word (in text fields) |

## Quick Reference Card

```
╭─────────────────────────────────────────────╮
│           EnvelopeCLI Quick Keys            │
├─────────────────────────────────────────────┤
│  Navigation: j/k/h/l or Arrow keys          │
│  Select:     Enter or Space                 │
│  New:        a (add)                        │
│  Edit:       e                              │
│  Delete:     d                              │
│  Clear:      c                              │
│  Help:       ?                              │
│  Commands:   : or /                         │
│  Quit:       q                              │
╰─────────────────────────────────────────────╯
```
