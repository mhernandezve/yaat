# ADR-018: Copy/Render by Default, Symlinks Opt-In

**Status:** Accepted
**Date:** 2026-10-31
**Supersedes:** [ADR-002](0002-symlinks-by-default.md)

### Context
ADR-002 made symlinks the default. That was correct under the assumptions of the
time: a single host, static files, no secrets or templates, applications that
read a config without replacing it, a stable repository path, and no per-target
permissions. Those assumptions no longer hold.

YAAT now targets:

- Multiple hosts with different values per machine (email, paths, themes).
- VM-based workflows where the repository does not exist at the same path.
- Encrypted secrets that the application must read decrypted.
- Templated files that must be rendered before use.
- Targets with specific permissions or ownership (`0600`, root-owned files).

A symlink cannot represent any of these:

- There is no place to render variables.
- Encrypted files cannot be symlinked and read by the application.
- Applications that save through `temp + rename` replace the link, so the
  repository silently stops being the source of truth.
- Target permissions are governed by the linked file, not the link.
- Moving the repository breaks every link.
- Paths outside `$HOME` (system configs, containers) do not share the link.

Copy mode already exists through `symlink.enabled: false`, but it is global and
lacks drift detection, atomic writes, and a way to capture live edits back into
the repository.

### Decision
1. **Strategy per entry**, not a global flag: `link`, `copy`, or `render`
   (templates). The default is `copy`; `link` is opt-in for entries edited in
   place (for example `config/nvim`).
2. **Track the last applied content hash per target** to distinguish three
   cases: repository changed, target changed, or both. Overwrite only when the
   target is unchanged; otherwise require an explicit decision.
3. **Commands with clear semantics**:
   - `yaat apply`: render/copy into the target using an atomic write
     (`temp + rename`), preserving mode and executable bits, with backup.
   - `yaat diff`: show drift between repository and targets.
   - `yaat capture`: copy live edits back into the repository.
   - `yaat apply --prune`: remove targets no longer present in the manifest.
   - `yaat apply --materialize`: convert existing correct symlinks into copies.
4. **Templates** use a simple variable model (`vars` global, per host, and
   environment), with fail-closed behavior: a missing variable is an error, not
   an empty string.
5. **Secrets** are decrypted and rendered in memory to the target with `0600`.
   They are never written to the repository, to `rendered/` inside a public
   path, or to a world-readable location.
6. **Migration**: change the default in a major release, after `diff`,
   `apply`, and `capture` exist. `apply --materialize` converts links with
   backups and is idempotent.

### Consequences
- ✅ Supports multiple hosts, templates, secrets, permissions, and targets
  outside the repository path.
- ✅ Makes VM and container workflows viable; the repository no longer needs to
  be mounted at a fixed path.
- ✅ Applications that replace config files no longer break synchronization.
- ❌ Introduces state and drift between the repository and targets.
- ❌ Loses instant propagation; every change requires `apply`.
- ❌ Requires more code and tests: hashing, conflict handling, atomic writes,
  pruning, and backups.
- ❌ Copying large directories needs hashing or mtime checks to stay fast.

### Mitigation
- Keep `link` opt-in for directories that are edited constantly.
- `status` and `diff` report drift before applying.
- Backups and `--dry-run` remain the default safety net.
- An optional watcher can restore near-instant propagation where needed.

### Implementation Phases
1. Per-entry strategy in the manifest, plus content hashes and `diff`.
2. `apply` with atomic writes, modes, backups, and `--prune`.
3. `capture` and `--materialize`.
4. Templates (`vars`, per-host, fail-closed).
5. `age` encryption integrated with render.
6. Switch the default to `copy` and release a major version.
