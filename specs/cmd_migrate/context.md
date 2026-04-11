---
spec: cmd_migrate.spec.md
---

## Domain Context

spec-sync v4.0.0 is a major version break that restructures how project metadata is stored. The migration command is the primary upgrade path — without it, users would need to manually move files, edit frontmatter, and create directory structures.

## Key Terminology

- **3.x layout**: Config at root (`specsync.json`, `specsync-registry.toml`), lifecycle history embedded in spec frontmatter as `lifecycle_log` YAML field
- **4.0 layout**: All metadata under `.specsync/` directory — `config.json`, `registry.toml`, `lifecycle/{module}.json`, `changes/`, `archive/`
- **Migration step**: An atomic unit of work with a check function (is this done?) and an apply function (do it)
- **Step status**: `Done` (skip), `Pending` (needs work), `Partial` (previous run crashed mid-step — fix forward)

## Design Rationale

### Why `.specsync/` directory?

Root-level config files don't scale. Projects with 100+ specs accumulate lifecycle history, change records, and archived specs that clutter the project root. A dedicated directory keeps metadata organized and allows `.gitignore` control over which parts are committed.

### Why no symlink backwards-compat?

Symlinks are fragile on Windows (`core.symlinks = false` in git), confuse some CI tools, and create surprising behavior. A clear "run migrate" error is better than a symlink that silently breaks on some platforms.

### Why step-based (not atomic swap)?

True atomic directory swaps aren't portable across filesystems. Step-based with idempotent checks is more robust — if migration crashes, re-running picks up where it left off. The backup provides the safety net.

### Why extract lifecycle_log from frontmatter?

Frontmatter is for spec metadata (module, version, status, dependencies). Lifecycle history grows unboundedly and is operational data, not spec content. Externalizing it keeps specs clean and allows lifecycle queries without parsing every spec file.

### Modeled after `cargo fix --edition`

Rust's edition migration is the gold standard: automatic, non-interactive, test-driven validation after migration, clear handling of edge cases that can't be auto-fixed. We adopt the same philosophy: migrate everything possible, flag what can't be migrated, validate the result.
