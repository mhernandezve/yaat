# ADR-012: Dry-Run Mode

**Status:** Accepted
**Date:** 2026-03-27

### Context
Users need to preview changes before applying.

### Decision
Implement `--dry-run` flag for `sync` and `backup` commands.

### Implementation
Check flag at each filesystem operation, log "would do X" instead of doing it.
