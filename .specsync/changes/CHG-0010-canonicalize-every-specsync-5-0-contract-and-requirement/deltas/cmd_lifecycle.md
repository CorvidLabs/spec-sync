## ADDED

### REQUIREMENT REQ-cmd-lifecycle-001

The lifecycle command SHALL enforce valid spec maturity transitions and SHALL preserve deterministic text and structured output.

Acceptance Criteria
- `promote`/`demote` use `SpecStatus::next()`/`prev()`; `set` accepts any valid status; all validate via `can_transition_to()` unless `--force`.
- Guard evaluation checks `min_score`, `require_sections`, and staleness (`no_stale`/`stale_threshold`), matching both specific (`from→to`) and wildcard (`*→to`) keys in either Unicode (`→`) or ASCII (`->`) form.
- A blocked transition (invalid jump or failed guard) prints the failures and exits 1; `--force` overrides both.
- When `track_history` is enabled, successful transitions append a dated `lifecycle_log` entry to the spec frontmatter.
- `auto-promote` advances only specs whose next transition passes guards (or, with `--dry-run`, reports what would change without writing).
- `enforce` exits non-zero when any selected check (`require_status`, `check_allowed`, `check_max_age`) finds a violation; `status`, `history`, and `guard` honor JSON output.

## MODIFIED

### SPEC SECTION Public API

**Exported Structs**

| Type | Description |
|------|-------------|
| `GuardResult` | Result of evaluating transition guards — `passed: bool` and `failures: Vec<String>` |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_promote` | `root: &Path, spec_filter: &str, format: OutputFormat, force: bool` | `()` | Advance a spec to its next lifecycle status (draft→review→active→stable) |
| `cmd_demote` | `root: &Path, spec_filter: &str, format: OutputFormat, force: bool` | `()` | Move a spec back to its previous lifecycle status |
| `cmd_set` | `root: &Path, spec_filter: &str, target_str: &str, format: OutputFormat, force: bool` | `()` | Set a spec to any valid status with transition validation |
| `cmd_status` | `root: &Path, spec_filter: Option<&str>, format: OutputFormat` | `()` | Display lifecycle status of one or all specs |
| `cmd_history` | `root: &Path, spec_filter: &str, format: OutputFormat` | `()` | Display lifecycle transition history for a spec |
| `cmd_guard` | `root: &Path, spec_filter: &str, target_str: Option<&str>, format: OutputFormat` | `()` | Evaluate and display guard results for a spec transition |
| `cmd_auto_promote` | `root: &Path, format: OutputFormat, dry_run: bool` | `()` | Scan all specs and promote any that pass transition guards; supports dry-run mode |
| `cmd_enforce` | `root: &Path, format: OutputFormat, require_status: bool, check_max_age: bool, check_allowed: bool` | `()` | CI enforcement: validate lifecycle rules across all specs, exit non-zero on violations |
| `evaluate_guards` | `root: &Path, spec_path: &Path, config: &SpecSyncConfig, from: &SpecStatus, to: &SpecStatus` | `GuardResult` | Evaluate all transition guards for a status change |

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `filter_specs` |
| parser | `parse_frontmatter`, `get_missing_sections` |
| scoring | `score_spec` |
| types | `SpecStatus`, `OutputFormat`, `SpecSyncConfig`, `LifecycleConfig`, `TransitionGuard` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync lifecycle` subcommands |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/git_utils/git_utils.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
