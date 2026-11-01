# ADR-011: Cross-Platform with `dirs` Crate

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
