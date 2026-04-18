# YAAT Roadmap / Pending Tasks

See [DESIGN_DECISIONS.md](./DESIGN_DECISIONS.md) for architectural decisions and rationale.

---

## ✅ Completados recientemente

- [x] Completely rewrite README for current version
- [x] Remove Windows support (Unix-only)
- [x] Add `yaat update` command with `--ask-unknown` flag
- [x] Implement `.yaatignore` support
- [x] Separate `config_dirs` and `home_files` structure

---

## v0.1 — Release público

### Inmediato (sin código)

- [ ] Agregar `LICENSE` MIT al repo
- [ ] Agregar `license = "MIT"` en `Cargo.toml`
- [ ] Quitar Windows de la matrix en CI (`os: [ubuntu-latest, macos-latest]`)

### CI / Tooling

- [ ] Agregar `cargo test --verbose` al workflow de CI
- [ ] Configurar **lefthook** para conventional commits locales
  - Validación via regex en `lefthook.yml`, sin dependencia de Node
- [ ] Agregar job de **commitlint** al CI (`wagoid/commitlint-github-action`)
- [ ] Configurar **Release Please** para versionamiento automático
  - `release-type: rust`
  - Genera `CHANGELOG.md` y actualiza versión en `Cargo.toml`
- [ ] Configurar **cargo-dist** para release workflow
  - Targets: `x86_64-linux`, `aarch64-linux`, `x86_64-macos`, `aarch64-macos`
  - Genera installer script para usar en bootstrap

### Convención de commits

Scopes definidos: `cli`, `sync`, `manifest`, `encrypted`, `ci`, `deps`, `bootstrap`

Formato: `tipo(scope): descripción`

```
feat(cli): add update command
fix(sync): resolve symlink conflict on macOS
chore(ci): add release-please workflow
```

### Distribución

- [ ] Binarios precompilados via GitHub Releases (manejado por cargo-dist)
- [ ] Installer script generado por cargo-dist para usar en bootstrap

### Bootstrap repo (`miguel/bootstrap`)

- [ ] Crear repo `bootstrap`
- [ ] Script único que:
  1. Descarga e instala YAAT via installer script
  2. Corre `yaat init <repo-url>`
  3. Limpia archivos temporales
- [ ] Un solo comando de entrada:
```bash
curl -sL https://raw.githubusercontent.com/miguel/bootstrap/main/bootstrap.sh | bash -s work-laptop
```

### README

- [ ] Quitar referencias a Windows (plataformas soportadas + sección de symlinks)
- [ ] Actualizar manifest de ejemplo — reemplazar `include` por `config_dirs` y `home_files`
- [ ] Remover "Symlink or copy" de features — solo symlinks
- [ ] Agregar `update` command en Quick Start y Project Structure (`commands/update.rs`)
- [ ] Quitar referencias a `dotfiles-alt` (repo no existe)
- [ ] Agregar sección de conventional commits en Contributing
- [ ] Actualizar Installation con installer script de cargo-dist
- [ ] Agregar mención al bootstrap repo

### Tests automatizados

- [ ] Integration tests for `backup` command
- [ ] Integration tests for `sync` command
- [ ] Unit tests for yaat.yaml parsing
- [ ] Tests for path resolution functions
- [ ] Tests for symlink creation and validation

### Manejo de edge cases

- [ ] Handle broken symlinks (symlink pointing to non-existent file)
- [ ] Handle conflicts (file exists and differs from repo)
- [ ] Handle permission issues (configs with 600 vs 644 permissions)
- [ ] Handle disk full scenarios
- [ ] Handle concurrent modifications

### Known configs

- [ ] Estructurar como `KnownConfig` struct con categorías para facilitar contribuciones externas:
```rust
pub struct KnownConfig {
    pub name: &'static str,
    pub config_path: &'static str,
    pub category: Category,
}

pub enum Category {
    Shell, Editor, Terminal, WM, Bar, Notification, Other,
}
```

---

## v0.2 — Post-release

### Encrypted con age

- [ ] Agregar crate `age` como dependencia (sin dependencia de binario externo)
- [ ] Agregar sección `encrypted` al manifest:
```yaml
encrypted:
  enabled: false
  key: ~/.ssh/id_ed25519
  files:
    - .gitconfig
    - .ssh/config
```
- [ ] Crear carpeta `encrypted/` en el repo (archivos `.age`)
- [ ] Crear carpeta `rendered/` gitignoreada
- [ ] Pipeline: `archivo.age` → decrypt → `rendered/` → symlink → `~/`
- [ ] Casos de uso principales a documentar: `.gitconfig` y `.ssh/config`
- [ ] Comandos:
  - `yaat add .gitconfig --encrypt`
  - `yaat encrypt .gitconfig` (sobre archivo ya trackeado)

### Host-specific configs

- [ ] Activar sección comentada del manifest
- [ ] Permite exclusiones por host lógico:
```yaml
hosts:
  work-laptop:
    exclude:
      - hypr/desktop-specific.conf
```

### Mejoras de UX

- [ ] `yaat doctor` command — Verificar salud del repo
  - Check for broken symlinks
  - Check for files in repo that don't exist in system
  - Check for configs in system not tracked in repo
  - Check for permission mismatches
  - Verify git repository integrity
  - Suggest fixes for found issues
- [ ] `yaat clean` command — Limpiar symlinks rotos o configs desinstaladas
  - Remove broken symlinks
  - Remove configs that have been uninstalled
  - Clean up old backup files (*.backup)
  - Option to dry-run before cleaning
- [ ] Better output
  - Add colors to output (green=OK, red=error, yellow=warning)
  - Add progress bars for long operations
  - Add summary at end (X added, Y ignored, Z errors, W skipped)
- [ ] Shell completions
  - Bash completion
  - Zsh completion
  - Fish completion

---

## v0.3 — TUI

- [ ] Implementar como subcomando `yaat tui` (un solo binario)
- [ ] Dependencias: `ratatui` + `crossterm`
- [ ] Dos vistas principales:
  - **Status interactivo** — lista de archivos con estado (synced / not synced / modified), acciones directas (`s` sync, `b` backup, `a` add)
  - **Editor de manifest** — gestión visual de `config_dirs`, `home_files`, `encrypted`, exclusiones sin editar YAML a mano
- [ ] Reutiliza lógica existente de `config.rs` y `commands/` — solo capa de presentación

---

## 📝 Documentation (ongoing)

- [ ] Add man page (`man yaat`)
- [ ] Add `--help` examples for each command
- [ ] Video tutorial / GIF demos
- [ ] Migration guide from other tools (yadm, chezmoi, stow)

---

## 🟢 Nice-to-Have (Future Ideas)

- [ ] Templates — `yaat template save/load` para diferentes setups
- [ ] Hooks — Scripts pre/post backup/sync
- [ ] Global configuration (`~/.config/yaat/config.toml`)
- [ ] Web interface for browsing configs
- [ ] Cloud sync integration (optional, manual git is preferred)
- [ ] Config validation (lint yaat.yaml)
- [ ] Import/export from other formats

---

Last updated: 2026-04-18
