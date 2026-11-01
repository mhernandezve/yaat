# ADR-003: YAML over TOML for Configuration

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
