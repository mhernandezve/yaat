# YAAT (Yet Another Dotfiles Manager)

A Rust-based CLI tool for managing configuration files across multiple machines.

[![CI](https://github.com/mhernandezve/yaat/actions/workflows/ci.yml/badge.svg)](https://github.com/mhernandezve/yaat/actions/workflows/ci.yml)

## Features

- Multi-platform support (Linux, macOS, Windows)
- Git-based synchronization
- Host-specific configuration overrides
- Symlink management
- Simple CLI interface

## Installation

```bash
# Clone the repository
git clone https://github.com/mhernandezve/yaat.git
cd yaat

# Build and install
cargo build --release

# The binary will be available at target/release/yaat
```

## Usage

```bash
# Initialize a new dotfiles repository
yaat init <path>

# Add a configuration file to tracking
yaat add <file>

# Sync configurations from repository to system
yaat sync

# Check status
yaat status
```

## Development

```bash
# Build the project
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings
```

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
