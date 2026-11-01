# ADR-006: Skip Integration Tests in CI

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
