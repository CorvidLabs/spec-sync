# Migrating SpecSync Projects

## Adopting the 5.0 Verified SDD Lifecycle

This section is the 5.0 recipe. It is not the 6.0 default. Upgrading a 4.x project to 5.0 did not silently enable lifecycle enforcement. Preview and explicitly adopt it:

```bash
specsync change adopt --dry-run
specsync change adopt
specsync agents install
specsync check --strict
```

On 5.0, adoption wrote `.specsync/sdd.json`, detected an explicit test command, reported canonical requirement companions that need stable IDs, and preserved existing companions without making empty files mandatory. If `openspec/` or `.specify/` was detected, active/canonical artifacts were imported with provenance while historical archives remained in place. Update GitHub Actions from `CorvidLabs/spec-sync@v4` to `@v5` after that 5.0 adoption.

## Migrating from 5.x to 6.0

Upgrading the binary does **not** flip an existing `.specsync/sdd.json` off. Omitted `enabled` and `require_change_for_meaningful_files` keys still deserialize as **on** (`SddPolicy::default()` is fail-closed). No `specsync migrate` run is needed for the 5.x → 6.0 layout: `.specsync/` keeps its 5.x files, the existing `.specsync/version` stamp is left untouched (6.0 reads it only to detect a pre-4.0 project), and `specsync migrate 5.0` exists solely to backfill 5.0.1-era reopening digests when `check` prints that remediation.

Existing 5.x consumers, in this order:

1. **Pin the 6.0.0 binary everywhere before resuming lifecycle work.** `cargo install --git https://github.com/CorvidLabs/spec-sync --tag v6.0.0 --locked specsync` (or the `specsync-linux-*` / `specsync-macos-*` release asset), then confirm `specsync --version` prints `specsync 6.0.0` on every developer machine, agent, hook, and CI runner. Linux and macOS only: 6.0 neither builds nor publishes a Windows binary (#735). Windows users run SpecSync under WSL or build from source.
2. **Update the GitHub Action to two explicit pins:** `uses: CorvidLabs/spec-sync@v6.0.0` with `version: '6.0.0'`. The `uses` ref pins the action code and `version` pins the binary it downloads. Drop any floating `@v5` ref or `version: latest`; pin the exact release rather than a floating major tag so CI runs the action code and binary you validated. The action now refuses a Windows runner.
3. **`specsync check` gates by default.** The default `enforcement` moved from `warn` to `strict`, so a bare `check` exits 1 on any validation error (warnings still pass unless `--strict` is given). Repositories with pre-existing spec errors start failing on the 6.0 binary: fix the specs, or keep the 5.x non-blocking behaviour with `enforcement = "warn"` in `.specsync/config.toml` or `--enforcement warn`.
4. **Finish, archive, or abandon every active workflow-v1 change and merge those PRs *before* running `specsync change adopt`** (#674). Workflow-v1 is **merge-then-archive**: `verify` → `accept` → merge the PR → `archive`. The workflow-v2 baseline (`.specsync/workflow-v2-baseline.json`) is written once — by `change adopt`, or by the first `change new` that mints a workflow-v2 change, whichever comes first — with a cutoff at the merge-base of your checkout and the remote default branch; the baseline is immutable and the cutoff never moves forward. A workflow-v1 change that merges after that cutoff can never be adopted — 6.0 recognises only v1 changes already present in the trusted history at the cutoff — and would have to be recreated as a workflow-v2 change. `adopt` refuses only when a v1 change in the *current* tree is missing from the cutoff history; it is **silent** on a committed still-active v1 that is already in that cutoff (it does not warn, archive, or migrate it). Run it from an up-to-date checkout of the default branch on which `specsync change list` shows no active changes.

   Literal close-out for each still-active workflow-v1 change (`<id>` from `specsync change list`):

   ```bash
   specsync change status <id>          # next_action names verify → accept → archive
   specsync change verify <id>
   specsync change accept <id>
   # merge the PR on GitHub (workflow-v1 is merge-then-archive)
   specsync change archive <id>
   ```

   `change status` on a v1 `Verifying` record names that sequence; it does not name the 6.0 `check` / `review` / `finalize` verbs.
5. **Run `specsync change adopt`.** On a policy that is already enabled it changes nothing in `sdd.json`, but it records the workflow-v2 baseline (if a `change new` has not already done so) and writes `.specsync/adoption-report.json`. **Commit both files** (they are the cutoff you will be judged against):

   ```bash
   specsync change adopt
   git add .specsync/workflow-v2-baseline.json .specsync/adoption-report.json
   git commit -m "Record the SpecSync 6.0 workflow-v2 baseline"
   ```

   From then on every new change uses the single 6.0 workflow (`approve` → `check --commit` → `review` → `ship`/`finalize` on the same PR) and is identified by a slug derived from its description, never a `CHG-NNNN` ordinal; 5.x `verify` / `accept` remain only as recovery for existing workflow-v1 records. Existing `CHG-` records stay readable as history, and `.specsync/change-sequence.json` is no longer allocated from.
6. **`specsync check` no longer walks SDD.** It does not inspect active changes, workspaces, or archives, even when the policy is enabled. Keep `specsync change audit` in CI if that gate is still wanted; it exits 1 on any active-workspace or living-spec error. The `CorvidLabs/spec-sync` GitHub Action runs `specsync check` (plus `lifecycle enforce --all` when `lifecycle-enforce` is set) and does not run the audit, so add it as its own step with a full-depth checkout. (The global `--strict` flag parses on `change audit` but has no effect there.)
7. **Neither `change check` nor `change verify` executes `sdd.json` `verification_commands`.** `change check` (workflow-v2) compares this change's specs to code in-process. `change verify` is the workflow-v1 recovery verb and the same in-process comparison — it does not spawn `cargo test` / `swift test` / the configured command list. Put those in CI.
8. **Fresh `init` writes SDD off** (`enabled: false`, `require_change_for_meaningful_files: false`, empty `verification_commands`). `specsync change adopt` is the on-switch: it writes an enabled policy when none exists, and on an existing file it flips **only** `enabled`. Path coverage stays where the author left it — so `init` then `adopt` gives you `enabled: true` with `require_change_for_meaningful_files: false`, which is **not** the 5.0 default. Set `require_change_for_meaningful_files: true` yourself if you want `change audit` (and archive) to require an active change for every meaningful-path edit. Hand-editing the policy changes its digest, so the bootstrap exemption `init` recorded for `.specsync/sdd.json` no longer covers the file and that edit needs an active change of its own. `adopt` fails closed on a policy it cannot parse and is a no-op on one already enabled. `enabled` governs `change audit` and archive-time path coverage; the `change new/approve/check/review/ship` verbs run either way.
9. **Lessons are no longer a `next_action` merge gate.** After `finalize` / `ship`, the archive still writes `lesson-bundle.md`; folding it into `context.md` is optional.

```bash
# existing 5.x repo: binary upgrade does not disable SDD
specsync --version               # 6.0.0 on every writer before touching a change
specsync change list             # close every active v1 change first (verify → accept → merge → archive)
specsync change adopt            # records the workflow-v2 baseline; slug IDs from here on
git add .specsync/workflow-v2-baseline.json .specsync/adoption-report.json
git commit -m "Record the SpecSync 6.0 workflow-v2 baseline"
specsync change audit            # keep this in CI if you still want the lifecycle gate
# cargo test / equivalent belongs in CI, not in change check or change verify

# new clone / fresh init
specsync init                    # SDD off
specsync change adopt            # on-switch; flips enabled only
git add .specsync/workflow-v2-baseline.json .specsync/adoption-report.json
git commit -m "Record the SpecSync 6.0 workflow-v2 baseline"
```

Pin the GitHub Action explicitly: `uses: CorvidLabs/spec-sync@v6.0.0` with `version: '6.0.0'` (the form the README and site examples use). Pre-releases were `@v6.0.0-rc.N`; a candidate is never reachable through a floating major ref, and the exact tag is the form to keep after release too. Do not treat a 5.0 `adopt` + `check --strict` sequence as an SDD gate on 6.0: `check` will not consult the change workflow.

### Validation, approval, and rollout boundaries

A successful check validates configured structure, exported API names, source mappings, dependency declarations, and supported schema rules. It does not prove that arbitrary natural-language requirements describe the implementation; reviewers and product tests establish those behaviors.

Approval digests bind recorded approval to content. The actor/reviewer label is not authenticated identity. Signed provenance and a required policy-verification check must be configured separately when identity or provenance enforcement is required; recording a signature or using soft mode alone is not that gate.

Coordinate the upgrade of all lifecycle writers to the selected 6.x version, including developer machines, agents, hooks, and CI. Older 5.x writers may reject slug-based changes or discard newer record fields. The 6.x downgrade checks detect damaged or downgraded evidence; they do not make mixed-version writes safe. Check `specsync --version` in each execution environment before resuming active work.

Pin the SpecSync executable separately from any Action or Trust wrapper. Use a wrapper revision that supports the selected binary version; adding an input to an older wrapper is not sufficient. This repository validates its candidate binary and supplies it to Trust through an explicit `specsync-version` and a runner-local `file://` mirror. Adopters should validate their own wrapper and binary source, and require provenance policy verification if it is a release gate.

For new changes, complete scope approval, implementation, scoped verification, and human implementation review, then run `review` and `ship` consecutively without an intervening commit. Commit the archive result on the same PR, wait for required checks, and merge only after every active change is archived. Legacy `verify` / `accept` recovery is for historical evidence. Workflow-v2 also supports audited `reopen` recovery for eligible accepted or archived records; resume through `check`, human review, `review`, and `finalize`. Recovery does not replace the normal archive-before-merge path.

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
