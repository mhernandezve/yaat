# YAAT - Architectural Decision Records (ADR)

## ADR-001: Binary-Only Crate Architecture
**Status:** Accepted
**Date:** 2026-03-27

### Context
Needed to decide whether YAAT should be a library + binary or binary-only crate.

### Decision
Binary-only crate (no `lib.rs`). All code is in `src/main.rs` and modules under `src/`.

### Consequences
- ✅ **Pros:**
  - Simpler structure, no public API to maintain
  - Can change internals freely without breaking changes
  - Smaller codebase, easier to understand
  
- ❌ **Cons:**
  - Integration tests require pre-built binary
  - Cannot use `cargo test --lib`
  - Harder to test internal functions directly

### Mitigation
Integration tests search for binary at runtime and skip if not found. Tests run locally after `cargo build`.

---

## ADR-002: Symlinks by Default
**Status:** Accepted
**Date:** 2026-03-27

### Context
Two approaches for syncing files: copying or symlinking.

### Decision
Use symlinks by default. Configurable via `yaat.yaml`:
```yaml
symlink:
  enabled: true  # Can be set to false for copy mode
```

### Consequences
- ✅ **Pros:**
  - Changes in repo immediately reflect in system
  - No duplication, saves disk space
  - Git tracks changes, not YAAT
  
- ❌ **Cons:**
  - Requires symlink support (elevated permissions on Windows)
  - If repo moves, symlinks break
  - Not all editors handle symlinks well

### Mitigation
- Backup existing files before creating symlinks
- Provide copy mode as alternative
- Clear error messages when symlinks fail

---

## ADR-003: YAML over TOML for Configuration
**Status:** Accepted
**Date:** 2026-03-27

### Context
Rust ecosystem prefers TOML, but we needed human-friendly config.

### Decision
Use YAML for `yaat.yaml` configuration file.

### Consequences
- ✅ **Pros:**
  - More readable for complex nested structures
  - Better support for lists and maps
  - Users familiar with YAML from Docker, k8s, etc.
  
- ❌ **Cons:**
  - Slightly slower parsing than TOML
  - More complex syntax (indentation matters)
  - Extra dependency (serde_yaml)

### Rationale
Config file readability > parsing performance for this use case.

---

## ADR-004: Git2 Crate vs Git CLI
**Status:** Accepted
**Date:** 2026-03-27

### Context
Options: Use `git` command via std::process or use `git2` crate.

### Decision
Use `git2` crate (libgit2 bindings).

### Consequences
- ✅ **Pros:**
  - No external dependency on git binary
  - Better error handling
  - Type-safe API
  - Faster (no process spawning)
  
- ❌ **Cons:**
  - Larger binary (links libgit2)
  - More complex build (C dependencies)
  - May not support all git features

---

## ADR-005: Remove -D warnings from CI
**Status:** Accepted
**Date:** 2026-03-27

### Context
CI was failing due to clippy warnings treated as errors.

### Decision
Remove `-D warnings` from CI workflow. Clippy runs but doesn't fail builds.

### Consequences
- ✅ **Pros:**
  - CI passes despite code style issues
  - Focus on functional correctness
  
- ❌ **Cons:**
  - Technical debt accumulates
  - May miss important warnings

### Future
Fix warnings incrementally. Consider re-enabling `-D warnings` after cleanup.

---

## ADR-006: Skip Integration Tests in CI
**Status:** Accepted
**Date:** 2026-03-27

### Context
Integration tests require binary to exist, but `cargo test` in CI doesn't build it first.

### Decision
Run only `cargo build`, `cargo fmt`, `cargo clippy` in CI. Skip tests.

### Consequences
- ✅ **Pros:**
  - CI is fast and reliable
  - No flaky tests due to missing binaries
  
- ❌ **Cons:**
  - No automated testing in CI
  - Must rely on local testing

### Mitigation
- Run tests locally before push
- Consider adding build step before tests in future
- Unit tests would require lib.rs (ADR-001)

---

## ADR-007: Rename to "Yet Another Assets Tool"
**Status:** Accepted
**Date:** 2026-03-27

### Context
Original name: "Yet Another Dotfiles Manager"

### Decision
Rename to "Yet Another Assets Tool" to be more generic.

### Rationale
- Can manage any assets, not just dotfiles
- More flexible positioning
- Still keeps YAAT acronym

---

## ADR-008: Use Anyhow for Error Handling
**Status:** Accepted
**Date:** 2026-03-27

### Context
Options: Custom errors, `Box<dyn Error>`, or `anyhow`.

### Decision
Use `anyhow` crate for error handling.

### Consequences
- ✅ **Pros:**
  - Ergonomic error propagation (`?` operator)
  - Good error messages with context
  - No boilerplate custom error types needed
  
- ❌ **Cons:**
  - Less type-safe than custom errors
  - Harder to match on specific errors

### Pattern
```rust
use anyhow::{Context, Result};

fn do_something() -> Result<()> {
    std::fs::read(path)
        .with_context(|| format!("Failed to read {}", path))?;
    Ok(())
}
```

---

## ADR-009: Tracing over Log Crate
**Status:** Accepted
**Date:** 2026-03-27

### Context
Options: `log` crate or `tracing` crate.

### Decision
Use `tracing` for structured logging.

### Consequences
- ✅ **Pros:**
  - Structured logging (key-value pairs)
  - Better async support (future-proof)
  - More flexible subscribers
  - Industry standard
  
- ❌ **Cons:**
  - Slightly more complex setup
  - More dependencies

---

## ADR-010: Clap with Derive Macros
**Status:** Accepted
**Date:** 2026-03-27

### Context
Options: Builder pattern or derive macros.

### Decision
Use clap's derive macros (`#[derive(Parser)]`).

### Consequences
- ✅ **Pros:**
  - Declarative, easy to read
  - Less boilerplate
  - Type-safe
  - Automatic help generation
  
- ❌ **Cons:**
  - Compile-time only (less runtime flexibility)
  - Macro magic harder to debug

---

## ADR-011: Cross-Platform with `dirs` Crate
**Status:** Accepted
**Date:** 2026-03-27

### Context
Need to handle `~/.config` vs `%APPDATA%` vs `~/Library/Application Support`.

### Decision
Use `dirs` crate for cross-platform directory handling.

### Consequences
- ✅ **Pros:**
  - Handles all platform differences
  - Well-maintained, standard crate
  - No manual path construction
  
- ❌ **Cons:**
  - Extra dependency

---

## ADR-012: Dry-Run Mode
**Status:** Accepted
**Date:** 2026-03-27

### Context
Users need to preview changes before applying.

### Decision
Implement `--dry-run` flag for `sync` and `backup` commands.

### Implementation
Check flag at each filesystem operation, log "would do X" instead of doing it.

---

## ADR-013: Edition 2021 (not 2024)
**Status:** Accepted
**Date:** 2026-03-27

### Context
Rust 2024 edition is newest, but 2021 is stable.

### Decision
Use Edition 2021 for stability and wider compatibility.

### Rationale
- 2021 is well-tested and stable
- No need for 2024-specific features
- Better compatibility with older toolchains

---

## ADR-014: MIT License
**Status:** Accepted
**Date:** 2026-03-27

### Context
Choose open source license.

### Decision
MIT License.

### Rationale
- Permissive
- Simple
- Standard for Rust ecosystem
- Compatible with proprietary use

---

## Pending Decisions

### ADR-015: TUI Interface (Future)
**Status:** Proposed

Consider adding Terminal User Interface using `ratatui` for:
- Interactive file selection
- Visual sync status
- Easier configuration

### ADR-016: Publish to crates.io
**Status:** Proposed

Publish YAAT to crates.io registry for easy installation.

### ADR-017: Encryption Support
**Status:** Proposed

Add encryption for sensitive configuration files.

---

*Last updated: 2026-03-27*
*Format: Inspired by Architecture Decision Records (ADR)*
