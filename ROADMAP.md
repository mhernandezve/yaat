# YAAT Roadmap / Pending Tasks

## ✅ Recently Completed

- [x] Completely rewrite README for current version
- [x] Remove Windows support (Unix-only)
- [x] Add `yaat update` command with `--ask-unknown` flag
- [x] Implement `.yaatignore` support
- [x] Separate `config_dirs` and `home_files` structure

---

## 🔴 Critical (Before Production Use)

### 1. Automated Tests
No tests currently exist. Minimum needed:
- [ ] Integration tests for `backup` command
- [ ] Integration tests for `sync` command
- [ ] Unit tests for yaat.yaml parsing
- [ ] Tests for path resolution functions
- [ ] Tests for symlink creation and validation

### 2. Edge Case Error Handling
- [ ] Handle broken symlinks (symlink pointing to non-existent file)
- [ ] Handle conflicts (file exists and differs from repo)
- [ ] Handle permission issues (configs with 600 vs 644 permissions)
- [ ] Handle disk full scenarios
- [ ] Handle concurrent modifications

---

## 🟡 Important (UX Improvements)

### 3. `yaat doctor` Command
Verify repository health:
- [ ] Check for broken symlinks
- [ ] Check for files in repo that don't exist in system
- [ ] Check for configs in system not tracked in repo
- [ ] Check for permission mismatches
- [ ] Verify git repository integrity
- [ ] Suggest fixes for found issues

### 4. `yaat clean` Command
Cleanup operations:
- [ ] Remove broken symlinks
- [ ] Remove configs that have been uninstalled
- [ ] Clean up old backup files (*.backup)
- [ ] Option to dry-run before cleaning

### 5. Better Output/UX
- [ ] Add colors to output (green=OK, red=error, yellow=warning)
- [ ] Add progress bars for long operations
- [ ] Add summary at end (X added, Y ignored, Z errors, W skipped)
- [ ] Add timestamps to log messages
- [ ] Quieter mode (less verbose by default)

### 6. Global Configuration
Support `~/.config/yaat/config.toml`:
- [ ] Default host name
- [ ] Preferred editor for editing yaat.yaml
- [ ] Enable/disable colors
- [ ] Default dry-run mode for safety
- [ ] Custom ignore patterns globally

---

## 🟢 Nice-to-Have (Advanced Features)

### 7. Templates
- [ ] `yaat template save <name>` - Save current setup as template
- [ ] `yaat template load <name>` - Load a saved template
- [ ] `yaat template list` - List available templates
- [ ] Built-in templates: minimal, work, desktop, server

### 8. Hooks
- [ ] Pre-backup hooks (scripts to run before backup)
- [ ] Post-backup hooks
- [ ] Pre-sync hooks
- [ ] Post-sync hooks
- [ ] Configurable per-host or global

### 9. Shell Completions
- [ ] Bash completion
- [ ] Zsh completion
- [ ] Fish completion

### 10. Pre-built Binaries
- [ ] GitHub Actions workflow for releases
- [ ] Pre-compiled binaries for Linux (x86_64, ARM64)
- [ ] Pre-compiled binaries for macOS (Intel, Apple Silicon)
- [ ] Homebrew formula
- [ ] AUR package

---

## 📝 Documentation

- [ ] Add man page (`man yaat`)
- [ ] Add `--help` examples for each command
- [ ] Video tutorial / GIF demos
- [ ] Migration guide from other tools (yadm, chezmoi, stow)

---

## 🔮 Future Ideas

- [ ] GUI/TUI interface (using ratatui or similar)
- [ ] Web interface for browsing configs
- [ ] Encrypted configs support
- [ ] Cloud sync integration (optional, manual git is preferred)
- [ ] Config validation (lint yaat.yaml)
- [ ] Import/export from other formats

---

## Priority Order

1. **Tests** - Essential for confidence in changes
2. **Edge case handling** - Prevents data loss
3. **`yaat doctor`** - Helps users debug issues
4. **Better output** - Improves daily UX
5. **Shell completions** - Small effort, big UX win
6. **Pre-built binaries** - Easier installation
7. **Templates** - Power user feature
8. **Hooks** - Advanced automation

---

Last updated: 2026-04-17
