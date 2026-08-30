# Migrating SpecSync Projects

## Adopting the 5.0 Verified SDD Lifecycle

Upgrading the binary does not silently enable lifecycle enforcement in an existing project. Preview and explicitly adopt it:

```bash
specsync change adopt --dry-run
specsync change adopt
specsync agents install
specsync check --strict
```

Adoption writes `.specsync/sdd.json`, detects an explicit test command, reports canonical requirement companions that need stable IDs, and preserves existing companions without making empty files mandatory. If `openspec/` or `.specify/` is detected, active/canonical artifacts are imported with provenance while historical archives remain in place. Update GitHub Actions from `CorvidLabs/spec-sync@v4` to `@v5` after adoption.

New projects initialized by 6.0 write SDD **off**. `specsync check` is the product. Enable the change workflow with `specsync change adopt` if you want it.

## Migrating to SpecSync v4.0.0

This guide covers upgrading from SpecSync 3.x to 4.0.0.

## Breaking Changes

### Directory structure: `.specsync/` replaces root config files

SpecSync v4 moves all configuration and metadata into a `.specsync/` directory:

| v3.x Location | v4.0.0 Location |
|---|---|
| `specsync.json` | `.specsync/config.toml` |
| `.specsync.toml` | `.specsync/config.toml` |
| `specsync-registry.toml` | `.specsync/registry.toml` |
| _(in-spec frontmatter)_ | `.specsync/lifecycle/*.json` |
| _(not tracked)_ | `.specsync/changes/` |
| _(not tracked)_ | `.specsync/archive/` |
| _(not tracked)_ | `.specsync/version` |

**Impact**: Any CI scripts, Makefiles, or tool configs that reference `specsync.json` or `specsync-registry.toml` at the repo root must be updated.

### Config format: JSON to TOML

The config file is now TOML (`config.toml`) instead of JSON (`specsync.json`). The `specsync migrate` command converts automatically. Config resolution order is:

```
.specsync/config.toml → .specsync/config.json → .specsync.toml → specsync.json → defaults
```

v3 config files still work as a fallback, but new features will only be added to the v4 format.

### `lifecycle_log` removed from spec frontmatter

The `lifecycle_log` field in spec YAML frontmatter has been extracted into standalone JSON files under `.specsync/lifecycle/`. The `specsync migrate` command handles this automatically.

**Before (v3)**:
```yaml
---
module: auth
lifecycle_log:
  - "2026-04-01: draft → review"
  - "2026-04-05: review → stable"
---
```

**After (v4)**:
```yaml
---
module: auth
---
```

With a corresponding `.specsync/lifecycle/auth.json` file containing the extracted history.

### GitHub Action: `@v3` → `@v4`

Update your workflow files:

```yaml
# Before
- uses: CorvidLabs/spec-sync@v3

# After
- uses: CorvidLabs/spec-sync@v4
```

### New action input: `lifecycle-enforce`

The GitHub Action now supports `lifecycle-enforce: 'true'` to run `specsync lifecycle enforce --all` in CI.

## How to Migrate

### Step 1: Update the binary

```bash
cargo install specsync    # or download from GitHub Releases
```

### Step 2: Preview the migration

```bash
specsync migrate --dry-run
```

This shows what will change without modifying any files.

### Step 3: Run the migration

```bash
specsync migrate
```

This will:
1. Detect your 3.x project structure
2. Back up existing config to `.specsync/backup-3x/` (with manifest)
3. Create `.specsync/` directory structure (`lifecycle/`, `changes/`, `archive/`)
4. Convert `specsync.json` → `.specsync/config.toml`
5. Move `specsync-registry.toml` → `.specsync/registry.toml`
6. Extract `lifecycle_log` entries from spec frontmatter into `.specsync/lifecycle/*.json`
7. Clean `lifecycle_log` from spec frontmatter
8. Create `.specsync/.gitignore`
9. Scan for cross-project references
10. Stamp `.specsync/version` with `4.0.0`

### Step 4: Verify

```bash
specsync check
```

### Step 5: Commit the changes

```bash
git add .specsync/ specs/
git commit -m "chore: migrate to specsync v4.0.0"
```

### Step 6: Update CI

Replace `@v3` with `@v4` in your GitHub Actions workflows.

## Migration properties

- **Idempotent**: Safe to run multiple times. Already-completed steps are skipped.
- **Atomic**: Uses preflight checks before applying changes.
- **Reversible**: Original files are backed up to `.specsync/backup-3x/` with a `manifest.json`.
- **`--no-backup`**: Skip the backup step if you're confident (or re-running after a partial migration).
- **`--dry-run`**: Preview all changes without writing to disk.
- **JSON output**: Use `--format json` for structured output in scripts.

## New in v4.0.0

### Spec Lifecycle Management

Full lifecycle tracking for specs: `draft → review → stable → deprecated → archived`.

```bash
specsync lifecycle status              # See all specs' lifecycle status
specsync lifecycle promote auth        # Advance auth to next stage
specsync lifecycle guard auth          # Check if promotion guards pass
specsync lifecycle auto-promote        # Promote all eligible specs
specsync lifecycle enforce --all       # CI: fail if lifecycle rules violated
specsync lifecycle history auth        # View transition history
```

Configure transition guards in your config to enforce quality gates (e.g., minimum score, required sections, no warnings) before promotion.

### Change Records

`.specsync/changes/` tracks spec modifications over time, providing an audit trail separate from git history.

### Spec Archival

`specsync archive-tasks` moves completed tasks from `tasks.md` companion files. Retired specs can be moved to `.specsync/archive/`.

## FAQ

**Q: Can I stay on v3?**
Yes. v3.x config files are still read as a fallback. But new features (lifecycle enforcement, change records) require the v4 structure.

**Q: What if migration fails partway through?**
The migration is designed to handle partial state. Re-run `specsync migrate` and it will pick up where it left off. Your original files are in `.specsync/backup-3x/`.

**Q: Do I need to migrate all projects at once?**
No. Cross-project references (`depends_on: "owner/repo@module"`) work across v3 and v4 projects. But `specsync resolve --remote --verify` works best when all referenced projects are on v4.

---

# Embedded AI removal (5.0.0)

SpecSync 5.0 removes the embedded inference client, provider selection, model and endpoint configuration, API-key handling, automatic source transmission, and `aiCommand`/`SPECSYNC_AI_COMMAND` shell execution. Generation is now deterministic and local.

## What to do

1. Remove `aiProvider`, `aiModel`, `aiApiKey`, `aiBaseUrl`, `aiTimeout`, and `aiCommand` (or snake_case equivalents) from SpecSync configuration.
2. Replace `specsync generate --provider ... --model ...` with `specsync generate`.
3. Run `specsync agents install` or use `specsync mcp` so your coding agent can refine generated markdown using its own credentials and permissions.

Legacy key names are ignored with migration guidance; their values are never printed or executed.

The 4.4 provider system is historical only. Upgrading directly from 4.4 does not require translating providers: delete those settings and move enrichment to the coding-agent integration.
