# ADR-010: Clap with Derive Macros

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
