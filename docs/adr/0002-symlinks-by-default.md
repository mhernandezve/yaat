# ADR-002: Symlinks by Default

**Status:** Superseded by [ADR-018](0018-copy-render-by-default-symlinks-opt-in.md) (2026-10-31)
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
