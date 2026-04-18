# Design Decisions

This document records architectural and design decisions made for YAAT.

## Core Design

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Sync Mechanism** | Symlinks only | Simpler, instant updates, no duplication |
| **Platforms** | Unix only (Linux, macOS) | Primary use case, simplifies codebase |
| **Configuration Format** | YAML (yaat.yaml) | Human-readable, widely supported |
| **Structure** | `config_dirs` + `home_files` | Clear separation: directories vs individual files |

## Security

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Encryption** | age crate (pure Rust) | No external binary dependency, modern crypto |
| **Encrypted Storage** | `encrypted/` folder + `rendered/` (gitignored) | Clear separation, safe defaults |
| **SSH Key Reuse** | Use existing `~/.ssh/id_ed25519` | No new keys to manage |

## Workflow

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Hostname** | Logical role identifier | Not tied to machine hostname, portable configs |
| **Unknown Configs** | Interactive `--ask-unknown` flag | User control, discoverability |
| **Ignore List** | `.yaatignore` file | Persistent preferences across runs |
| **Installation** | Bootstrap repo pattern | Single-command setup, separation of concerns |

## Tooling

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Versioning** | Release Please + cargo-dist | Automated, industry standard |
| **Commits** | Conventional Commits | Automated changelog, clear history |
| **TUI** | Subcommand (`yaat tui`), not separate binary | Single tool, shared logic |
| **Licencia** | MIT | Permissive, widely accepted |

## Scope Exclusions

| Excluded | Reason |
|----------|--------|
| Windows support | Adds complexity, primary users are on Unix |
| Copy mode | Symlinks are sufficient, copies add complexity |
| Cloud sync | Git is the sync mechanism, keeps it simple |
| Separate TUI binary | Extra maintenance, subcommand is cleaner |

## Conventional Commits Scopes

- `cli` - Command-line interface changes
- `sync` - Sync functionality
- `manifest` - yaat.yaml structure and parsing
- `encrypted` - Encryption features
- `ci` - Continuous integration
- `deps` - Dependencies
- `bootstrap` - Bootstrap scripts and repo

## Format

```
type(scope): description

feat(cli): add update command
fix(sync): resolve symlink conflict on macOS
chore(ci): add release-please workflow
```

---

Last updated: 2026-04-18
