# YAAT (Yet Another Assets Tool)

[![CI](https://github.com/mhernandezve/yaat/actions/workflows/ci.yml/badge.svg)](https://github.com/mhernandezve/yaat/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modern, cross-platform dotfiles manager written in Rust. YAAT helps you maintain a single source of truth for your configuration files across multiple machines.

## Features

- 🖥️ **Cross-platform**: Works on Linux, macOS, and Windows
- 🔗 **Git-based**: Version control for your configurations
- 🎯 **Host-specific configs**: Different settings per machine
- 🔗 **Symlink or copy**: Choose how to sync your files
- 🛡️ **Backup before sync**: Automatic backups prevent data loss
- 📝 **Dry-run mode**: Preview changes before applying
- 🔍 **Status tracking**: See what's synced and what's pending

## Installation

### From Source (requires Rust)

```bash
# Clone the repository
git clone https://github.com/mhernandezve/yaat.git
cd yaat

# Build release binary
cargo build --release

# Install to your system (optional)
cargo install --path .

# Or use directly
./target/release/yaat --help
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/mhernandezve/yaat/releases) (coming soon).

## Quick Start

### 1. Initialize a Dotfiles Repository

```bash
# Create a new dotfiles repository at ~/.dotfiles (default)
yaat init

# Or specify a custom location
yaat init ~/my-dotfiles

# Or clone from an existing remote repository
yaat init --clone https://github.com/username/dotfiles.git
```

This creates:
```
~/.dotfiles/
├── config/          # Files that go to ~/.config
├── home/            # Files that go to ~
├── hosts/           # Host-specific configurations
├── yaat.yaml        # YAAT configuration
├── .git/            # Git repository
└── .gitignore       # Default ignores
```

### 2. Add Configuration Files

```bash
# Add a file from ~/.config
yaat add ~/.config/kitty/kitty.conf

# Add with host-specific configuration
yaat add ~/.config/hypr/hyprland.conf --host desktop

# Files are automatically committed to git
```

### 3. Sync to Your System

```bash
# Preview what will be synced (dry-run)
yaat sync --dry-run

# Actually sync (creates symlinks by default)
yaat sync

# Sync only for a specific host
yaat sync --host desktop

# Output shows:
# - Files being backed up (existing files are renamed to .backup)
# - Symlinks being created
# - Summary of synced vs skipped files
```

### 4. Backup Current System

```bash
# Preview what will be backed up
yaat backup --dry-run

# Backup current configs to repository
yaat backup

# This copies files from ~/.config to the repo
# and creates a commit
```

### 5. Check Status

```bash
# Quick overview
yaat status

# Detailed status with tracked files
yaat status --verbose
```

Shows:
- Repository location
- Git status (modified/untracked files)
- Configuration settings
- Sync status (what's synced, pending, or diverged)

## Configuration

YAAT uses a `yaat.yaml` file in your dotfiles repository:

```yaml
# Repository path (relative to home or absolute)
repo_path: ~/.dotfiles

# Default hostname for syncing
default_host: my-laptop

# Files/directories to exclude globally
exclude:
  - .git
  - .gitignore
  - yaat.yaml
  - "*.tmp"

# Symlink settings
symlink:
  enabled: true
  backup: true  # Backup existing files before creating symlinks

# Host-specific configurations
hosts:
  desktop:
    files:
      - config/hypr/desktop-specific.conf
    exclude:
      - config/waybar/laptop-config
    env:
      MONITOR: DP-1
  
  laptop:
    files:
      - config/hypr/laptop-specific.conf
    exclude:
      - config/waybar/desktop-config
```

## How It Works

### Directory Structure

```
~/.dotfiles/
├── config/              # Maps to ~/.config
│   ├── kitty/
│   │   └── kitty.conf
│   ├── nvim/
│   │   └── init.lua
│   └── waybar/
│       └── config
├── home/                # Maps to ~
│   ├── .zshrc
│   ├── .bashrc
│   └── .gitconfig
└── hosts/
    ├── desktop/         # Desktop-specific overrides
    │   └── config/hypr/
    └── laptop/          # Laptop-specific overrides
        └── config/hypr/
```

### Sync Process

1. **From Repo to System** (`yaat sync`):
   - Reads files from `~/.dotfiles/config/` and `~/.dotfiles/home/`
   - Applies host-specific overrides from `~/.dotfiles/hosts/<hostname>/`
   - Backs up existing files (if enabled)
   - Creates symlinks or copies files to system locations

2. **From System to Repo** (`yaat backup`):
   - Scans `~/.config` and tracked home files
   - Copies modified files back to repository
   - Creates a git commit

## Advanced Usage

### Environment Variables

- `YAAT_REPO`: Override the default repository path
- `HOME`: Used to resolve `~` in paths
- `XDG_CONFIG_HOME`: Config directory (default: `~/.config`)

### Verbose Mode

Add `--verbose` or `-v` to any command for detailed output:

```bash
yaat --verbose sync
yaat init --verbose ~/my-dotfiles
```

### Dry Run

Preview changes without making them:

```bash
yaat sync --dry-run
yaat backup --dry-run
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_init_creates_repo
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy -- -D warnings

# Build release
cargo build --release
```

### Project Structure

```
src/
├── main.rs          # Entry point
├── cli.rs           # CLI argument parsing
├── config.rs        # Configuration management
├── git.rs           # Git operations wrapper
├── platform.rs      # Cross-platform utilities
└── commands/
    ├── init.rs      # Initialize repository
    ├── add.rs       # Add files to tracking
    ├── sync.rs      # Sync to system
    ├── backup.rs    # Backup to repository
    └── status.rs    # Show status
```

## Troubleshooting

### "Could not find YAAT repository"

Run `yaat init` to create a repository, or set `YAAT_REPO` environment variable:

```bash
export YAAT_REPO=/path/to/your/dotfiles
```

### "Path is not within home, config, or repo directories"

YAAT only tracks files in:
- `~/.config/` → stored in `config/`
- `~/` (home) → stored in `home/`
- Files already in the repo

### Symlink Issues on Windows

Windows requires Developer Mode or Administrator privileges to create symlinks. If symlinks fail, YAAT will fall back to copying files.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

Inspired by [yadm](https://github.com/TheLocehiliosan/yadm), [chezmoi](https://github.com/twpayne/chezmoi), and other dotfile managers.
