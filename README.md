# YAAT (Yet Another Assets Tool)

[![CI](https://github.com/mhernandezve/yaat/actions/workflows/ci.yml/badge.svg)](https://github.com/mhernandezve/yaat/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modern dotfiles manager written in Rust. YAAT helps you maintain a single source of truth for your configuration files across multiple machines using Git.

## Features

- 🖥️ **Unix support**: Works on Linux and macOS
- 🔗 **Git-based**: Version control for your configurations
- 🎯 **Host-specific configs**: Different settings per machine
- 📋 **Include list**: Selective backup (only what you want)
- 🔍 **Auto-detection**: Automatically finds your dotfiles during init
- 🔗 **Symlink or copy**: Choose how to sync your files
- 🛡️ **Backup before sync**: Automatic backups prevent data loss
- 📝 **Dry-run mode**: Preview changes before applying
- 🔍 **Status tracking**: See what's synced and what's pending
- 📦 **Package management**: Optional scripts for migrating installed packages

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

# Or use YAAT_REPO environment variable
export YAAT_REPO=~/my-dotfiles
yaat init
```

YAAT will auto-detect your configuration files and create an `include` list.

This creates:
```
~/.dotfiles/
├── config/          # Files that go to ~/.config
├── home/            # Files that go to ~
├── hosts/           # Host-specific configurations
├── packages/        # Package lists (optional)
├── scripts/         # Helper scripts (optional)
├── yaat.yaml        # YAAT configuration
├── .git/            # Git repository
└── .gitignore       # Default ignores
```

### 2. Review and Customize

Edit `yaat.yaml` to customize what gets backed up:

```yaml
repo_path: ~/.dotfiles

# Auto-detected configs (edit as needed)
include:
  - config/hypr/          # Window manager
  - config/waybar/        # Status bar
  - config/fish/          # Shell
  - config/kitty/         # Terminal
  - config/nvim/          # Editor
  # Remove items you don't want to track
  # Add items that weren't auto-detected

exclude:
  - .git
  - node_modules
  - target
  - .cache

symlink:
  enabled: true
  backup: true
```

### 3. Backup Current System

```bash
# Preview what will be backed up
yaat backup --dry-run

# Backup current configs to repository
yaat backup

# This only backs up files/directories in your include list
```

### 4. Sync to Another Machine

```bash
# Clone your dotfiles
git clone https://github.com/username/dotfiles.git ~/.dotfiles
cd ~/.dotfiles

# Preview sync
yaat sync --dry-run

# Actually sync (creates symlinks by default)
yaat sync

# Sync only for a specific host
yaat sync --host desktop
```

### 5. Check Status

```bash
# Quick overview
yaat status

# Detailed status with tracked files
yaat status --verbose
```

## Configuration

YAAT uses a `yaat.yaml` file in your dotfiles repository:

```yaml
# Repository path (relative to home or absolute)
repo_path: ~/.dotfiles

# Default hostname for syncing
default_host: my-laptop

# Files/directories to INCLUDE (if set, only these are backed up)
# Auto-populated during 'yaat init', edit as needed
include:
  - config/hypr/
  - config/waybar/
  - config/fish/
  - config/kitty/
  - config/nvim/
  - home/.bashrc

# Files/directories to exclude (applied within included paths)
exclude:
  - .git
  - .gitignore
  - yaat.yaml
  - node_modules
  - target
  - .cache
  - "*.tmp"

# Symlink settings
symlink:
  enabled: true
  backup: true  # Backup existing files before creating symlinks

# Host-specific configurations
hosts:
  desktop:
    files: []
    exclude: []
    env: {}
  
  laptop:
    files: []
    exclude: []
    env: {}
```

### Auto-Detected Configurations

During `yaat init`, YAAT automatically detects these common configurations:

- **Desktop**: hypr, waybar, walker, mako, omarchy
- **Terminals**: kitty, alacritty, ghostty, foot, wezterm
- **Editors**: nvim, vim, helix
- **Shells**: fish, zsh, bash
- **Multiplexers**: tmux, tmuxinator, zellij
- **Tools**: git, lazygit, btop, fastfetch, fzf
- **Input**: fcitx, fcitx5, ibus

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
├── hosts/               # Host-specific overrides
│   ├── desktop/
│   │   └── config/hypr/
│   └── laptop/
│       └── config/hypr/
└── yaat.yaml
```

### Sync Process

1. **From Repo to System** (`yaat sync`):
   - Reads files from `~/.dotfiles/config/` and `~/.dotfiles/home/`
   - Applies host-specific overrides from `~/.dotfiles/hosts/<hostname>/`
   - Backs up existing files (if enabled)
   - Creates symlinks or copies files to system locations

2. **From System to Repo** (`yaat backup`):
   - Only processes files/directories in `include` list
   - Skips symlinks (with warning)
   - Copies files back to repository
   - Creates a git commit

### Include List Behavior

- **If `include` is populated**: Only those files/directories are backed up
- **If `include` is empty**: Shows "Nothing to backup" message
- **Symlinks**: Always skipped (to avoid broken links)
- **Works with `exclude`**: You can include a directory but exclude specific files

## Helper Scripts

Place these in your dotfiles repository (not in YAAT itself):

### `apply-dotfiles`
Main entrypoint that orchestrates sync + runtime reload:
```bash
./scripts/apply-dotfiles
# Detects host, runs yaat sync, reloads desktop components
```

### `export-packages.sh` / `install-packages.sh`
For migrating installed packages:
```bash
# Export packages
./scripts/export-packages.sh

# Preview missing packages
./scripts/install-packages.sh --dry-run --only-missing

# Install missing packages
./scripts/install-packages.sh
```

See [dotfiles-alt](https://github.com/mhernandezve/dotfiles-alt) for a complete example.

## Environment Variables

- `YAAT_REPO`: Override the default repository path
- `DOTFILES_HOST`: Override auto-detected hostname
- `HOME`: Used to resolve `~` in paths
- `XDG_CONFIG_HOME`: Config directory (default: `~/.config`)

## Advanced Usage

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

### Working with YAAT_REPO

```bash
# Set temporarily for one command
YAAT_REPO=~/work-dotfiles yaat backup

# Or export for the session
export YAAT_REPO=~/work-dotfiles
yaat sync
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
├── main.rs           # Entry point
├── cli.rs            # CLI argument parsing
├── config.rs         # Configuration management
├── git.rs            # Git operations wrapper
├── known_configs.rs  # Auto-detection whitelist
├── platform.rs       # Cross-platform utilities
└── commands/
    ├── init.rs       # Initialize repository
    ├── add.rs        # Add files to tracking
    ├── sync.rs       # Sync to system
    ├── backup.rs     # Backup to repository
    └── status.rs     # Show status
```

## Troubleshooting

### "Could not find YAAT repository"

Run `yaat init` to create a repository, or set `YAAT_REPO` environment variable:

```bash
export YAAT_REPO=/path/to/your/dotfiles
```

### "No configs in include list, nothing to backup"

Your `include` list in `yaat.yaml` is empty. Add some configurations:

```yaml
include:
  - config/hypr/
  - config/fish/
```

Or run `yaat init` again to auto-detect.

### Package Installation

For package management scripts, ensure you have the appropriate helper:
- **Arch**: `yay` or `paru` for AUR packages
- **Debian/Ubuntu**: Standard `apt`

## Related Projects

- **Dotfiles Example**: [dotfiles-alt](https://github.com/mhernandezve/dotfiles-alt) - Complete example with scripts
- **Inspiration**: [yadm](https://github.com/TheLocehiliosan/yadm), [chezmoi](https://github.com/twpayne/chezmoi)

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
