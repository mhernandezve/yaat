# ADR-004: Git2 Crate vs Git CLI

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
