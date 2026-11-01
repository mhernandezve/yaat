# ADR-008: Use Anyhow for Error Handling

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
