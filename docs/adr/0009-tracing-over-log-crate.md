# ADR-009: Tracing over Log Crate

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
