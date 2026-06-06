# YAAT (Yet Another Assets Tool)

[![CI](https://github.com/mhernandezve/yaat/actions/workflows/ci.yml/badge.svg)](https://github.com/mhernandezve/yaat/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modern dotfiles manager written in Rust. YAAT helps you maintain a single source of truth for your configuration files across multiple machines using Git.

## Features

- 🖥️ **Unix support**: Works on Linux and macOS
- 🔗 **Git-based**: Version control for your configurations
- 🎯 **Host-specific configs**: Different settings per machine
- 📁 **Smart structure**: Separate `config_dirs` and `home_files`
- 🔍 **Auto-detection**: Automatically finds your dotfiles during init and update
- 🔄 **Update command**: Detect and add new configs as you install apps
- ❓ **Interactive mode**: Ask about unknown configs with `--ask-unknown`
- 🚫 **Smart ignore**: `.yaatignore` to persist skip decisions
- 🔗 **Symlink only**: Instant updates, no duplication
- 🛡️ **Backup before sync**: Automatic backups prevent data loss
- 📝 **Dry-run mode**: Preview changes before applying
- 🔍 **Status tracking**: See what's synced and what's pending

## Installation

### Pre-compiled Binaries (Recommended)

Install the latest release with the official installer:

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/mhernandezve/yaat/releases/latest/download/yaat-installer.sh | sh

# Or with Homebrew (macOS/Linux)
brew install mhernandezve/tap/yaat
```

The installer will download the appropriate binary for your platform and add it to your PATH.

Supported platforms:
- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)

### From Source (requires Rust)

```bash
# Install directly from GitHub
cargo install --git https://github.com/mhernandezve/yaat

# Or clone and build manually
git clone https://github.com/mhernandezve/yaat.git
cd yaat
cargo build --release

# Use directly
./target/release/yaat --help
```

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

# Clone from an existing remote repository
yaat init --clone https://github.com/username/dotfiles.git
```

**Note**: `yaat init` only creates new repositories. If the repository already exists, use `yaat update` instead.

YAAT will auto-detect your configuration files and create `config_dirs` and `home_files` lists.

This creates:
```
~/.dotfiles/
├── config/          # Directories that go to ~/.config (symlinked as directories)
├── home/            # Files that go to ~ (symlinked individually)
├── hosts/           # Host-specific configurations
├── yaat.yaml        # YAAT configuration
├── .gitignore       # Default ignores
└── .yaatignore      # Your personal ignore list (created with --ask-unknown)
```

### 2. Update to Detect All Configs

For the first time, run with `--ask-unknown` to interactively discover all your configs:

```bash
# Detect known configs automatically + ask about unknown ones
yaat update --ask-unknown

# Preview what would be added (dry-run)
yaat update --ask-unknown --dry-run
```

This will prompt you for each unknown config:
```
? aether (15 files) - Add to config_dirs? [y/N/i] y
  + Added: aether
? chromium (1247 files) - Add to config_dirs? [y/N/i] i
  + Added to .yaatignore
```

### 3. Review and Customize

Edit `yaat.yaml` to customize what gets backed up:

```yaml
repo_path: ~/.dotfiles

# Config directories in ~/.config/ (backed up as complete directories)
config_dirs:
  - hypr          # Window manager
  - waybar        # Status bar
  - fish          # Shell
  - kitty         # Terminal
  - nvim          # Editor
  # Remove items you don't want to track
  # Add items that weren't auto-detected

# Individual files in ~/ (backed up and symlinked individually)
home_files:
  - .bashrc
  - .gitconfig
  - .tmux.conf

exclude:
  - .git
  - node_modules
  - target
  - .cache

symlink:
  enabled: true
  backup: true
```

### 4. Backup Current System

```bash
# Preview what will be backed up
yaat backup --dry-run

# Backup current configs to repository
yaat backup

# This backs up all files in config_dirs and home_files
```

### 5. Sync to Another Machine

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

### 6. Check Status

```bash
# Quick overview
yaat status

# Detailed status with tracked files
yaat status --verbose
```

## Updating Your Repository

As you install new applications, YAAT can automatically detect and add their configurations.

### Detect New Known Configs

```bash
# Check for new configs from KNOWN_CONFIGS list and add them automatically
yaat update

# Example output:
# Detected 2 new known config directories
#   + btop
#   + ghostty
```

### Interactive Discovery

```bash
# Also ask about unknown configs interactively
yaat update --ask-unknown

# Preview without making changes
yaat update --ask-unknown --dry-run
```

### The .yaatignore File

When using `--ask-unknown`, you can press `i` to ignore a config permanently. These are stored in `.yaatignore`:

```bash
# ~/.dotfiles/.yaatignore
# Configs that won't be prompted in --ask-unknown

chromium
configstore
dconf
```

### Workflow Example

```bash
# Day 1: Initial setup
yaat init ~/.dotfiles
yaat update --ask-unknown  # Discover and add all your configs
yaat backup                # Copy everything to the repo

# Day 2: Install a new app (btop)
sudo pacman -S btop

# Day 3: YAAT automatically detects it
yaat update
# Output: "Detected 1 new known config: + btop"

# Day 4: Install something unknown
yaat update --ask-unknown
# ? new-app (12 files) - Add to config_dirs? [y/N/i]
```

## Configuration

YAAT uses a `yaat.yaml` file in your dotfiles repository:

```yaml
# Repository path (relative to home or absolute)
repo_path: ~/.dotfiles

# Default hostname for syncing
default_host: my-laptop

# Config directories in ~/.config/ to manage
# These are backed up as complete directories and symlinked as directory symlinks
config_dirs:
  - hypr
  - waybar
  - fish
  - kitty
  - nvim
  - tmux
  - mako

# Individual files in ~/ to manage
# These are backed up and symlinked individually
home_files:
  - .bashrc
  - .zshrc
  - .gitconfig
  - .tmux.conf

# Files/directories to exclude globally
exclude:
  - .git
  - .gitignore
  - yaat.yaml
  - "*.tmp"
  - "*.bak"
  - .cache

# Symlink settings
symlink:
  enabled: true
  backup: true  # Backup existing files before creating symlinks

# Host-specific configurations
hosts:
  desktop:
    exclude:
      - hypr/laptop-specific.conf
  laptop:
    exclude:
      - hypr/desktop-specific.conf
```

### Auto-Detected Configurations

During `yaat init` and `yaat update`, YAAT automatically detects these common configurations:

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
├── config/              # Maps to ~/.config (directories symlinked as a whole)
│   ├── kitty/
│   │   └── kitty.conf
│   ├── nvim/
│   │   └── init.lua
│   └── waybar/
│       └── config
├── home/                # Maps to ~ (individual files symlinked)
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

### Key Differences from Other Tools

| Feature | YAAT | Others |
|---------|------|--------|
| Config structure | `config_dirs` + `home_files` | Flat or mixed |
| Unknown configs | Interactive `--ask-unknown` | Manual only |
| Update workflow | `yaat update` command | Re-init or manual |
| Ignore list | `.yaatignore` file | Usually none |

### Sync Process

1. **From Repo to System** (`yaat sync`):
   - Reads files from `~/.dotfiles/config/` and `~/.dotfiles/home/`
   - `config_dirs` are symlinked as **directories** (`~/.config/hypr` → `~/.dotfiles/config/hypr`)
   - `home_files` are symlinked as **individual files** (`~/.bashrc` → `~/.dotfiles/home/.bashrc`)
   - Applies host-specific overrides from `~/.dotfiles/hosts/<hostname>/`
   - Backs up existing files (if enabled)

2. **From System to Repo** (`yaat backup`):
   - Processes files/directories in `config_dirs` and `home_files`
   - Skips symlinks (with warning)
   - Copies files back to repository
   - Creates a git commit

## Helper Scripts

For complete workflows (bootstrap new system, install packages, reload desktop components), see:

**[yaat-bootstrap](https://github.com/mhernandezve/yaat-bootstrap)** - Example scripts and templates for your dotfiles repository.

YAAT focuses solely on file synchronization. Helper scripts should be maintained in your own dotfiles repository.

## Environment Variables

- `YAAT_REPO`: Override the default repository path
- `DOTFILES_HOST`: Override auto-detected hostname
- `HOME`: Used to resolve `~` in paths
- `XDG_CONFIG_HOME`: Config directory (default: `~/.config`)

## Command Reference

### `yaat init [PATH]`
Initialize a new dotfiles repository.
- Creates directory structure
- Auto-detects known configs
- Fails if repository already exists

**Options:**
- `--clone <URL>` - Clone from existing remote repository instead of creating new

### `yaat update [OPTIONS] [PATH]`
Update an existing repository.
- Detects new known configs automatically
- `--ask-unknown`: Interactive prompt for unknown configs
- `--dry-run`: Preview changes without applying

### `yaat backup [OPTIONS]`
Backup system configs to repository.
- `--dry-run`: Preview what will be backed up
- `--host HOST`: Backup for specific host

### `yaat sync [OPTIONS]`
Sync repository configs to system.
- `--dry-run`: Preview what will be synced
- `--host HOST`: Sync for specific host

### `yaat status [OPTIONS]`
Show repository status.
- `--verbose`: Detailed status with file list

## Troubleshooting

### "Could not find YAAT repository"

Run `yaat init` to create a repository, or set `YAAT_REPO` environment variable:

```bash
export YAAT_REPO=/path/to/your/dotfiles
```

### "YAAT repository already exists"

`yaat init` can only create new repositories. To update an existing repository:

```bash
# Update existing repo (detects new known configs)
yaat update

# Update with interactive discovery
yaat update --ask-unknown
```

### "Nothing to backup"

Your `config_dirs` and `home_files` lists in `yaat.yaml` are empty. Run:

```bash
yaat update --ask-unknown
```

Or edit `yaat.yaml` manually to add configurations.

### Symlink Issues

YAAT requires Unix-based system (Linux or macOS). On Linux, ensure you have write permissions to `~/.config` and `~`.

### Package Installation

For package management scripts, ensure you have the appropriate helper:
- **Arch**: `yay` or `paru` for AUR packages
- **Debian/Ubuntu**: Standard `apt`

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

## Related Projects

- **Inspiration**: [yadm](https://github.com/TheLocehiliosan/yadm), [chezmoi](https://github.com/twpayne/chezmoi)

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Conventional Commits

This project follows [Conventional Commits](https://www.conventionalcommits.org/). Format: `type(scope): description`

**Types:** `feat`, `fix`, `chore`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `revert`

**Scopes:** `cli`, `sync`, `manifest`, `encrypted`, `ci`, `deps`, `bootstrap`, `licensing`

Examples:
```
feat(cli): add update command
fix(sync): resolve symlink conflict on macOS
chore(ci): add release-please workflow
```

### Setup

1. Fork the repository
2. Clone your fork (`git clone https://github.com/YOUR_USERNAME/yaat.git`)
3. Install lefthook for commit validation:
   ```bash
   # macOS/Linux via Homebrew
   brew install lefthook

   # Or download binary directly
   curl -fsSL https://github.com/evilmartians/lefthook/releases/latest/download/lefthook_$(uname -s)_$(uname -m) -o lefthook
   chmod +x lefthook
   sudo mv lefthook /usr/local/bin/

   # Install hooks in the repo
   lefthook install
   ```
4. Create a feature branch (`git checkout -b feature/amazing-feature`)
5. Make your changes and commit (`git commit -m 'feat(cli): add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## Acknowledgments

Inspired by [yadm](https://github.com/TheLocehiliosan/yadm), [chezmoi](https://github.com/twpayne/chezmoi), and other dotfile managers.

