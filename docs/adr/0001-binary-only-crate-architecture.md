# ADR-001: Binary-Only Crate Architecture

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
