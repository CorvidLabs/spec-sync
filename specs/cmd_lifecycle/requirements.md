---
spec: cmd_lifecycle.spec.md
---

## User Stories

- As a developer, I want to `promote`/`demote`/`set` a spec's status so that I can track its maturity (draft → review → active → stable → deprecated → archived).
- As a developer, I want `lifecycle status` to show one or all specs grouped by status so that I can see the project's maturity at a glance.
- As a developer, I want `lifecycle history` to show a spec's transition log so that I can audit how its status evolved.
- As a developer, I want `lifecycle guard` to dry-run guard evaluation for a transition so that I know why a promotion would be blocked before attempting it.
- As a team, we want `auto-promote` to advance every spec that already passes its guards so that maturing specs don't get stuck.
- As a CI operator, I want `enforce` to fail the build on lifecycle violations (missing status, disallowed status, specs stuck past `max_age`) so that lifecycle policy is enforced automatically.

## Acceptance Criteria

- `promote`/`demote` use `SpecStatus::next()`/`prev()`; `set` accepts any valid status; all validate via `can_transition_to()` unless `--force`.
- Guard evaluation checks `min_score`, `require_sections`, and staleness (`no_stale`/`stale_threshold`), matching both specific (`from→to`) and wildcard (`*→to`) keys in either Unicode (`→`) or ASCII (`->`) form.
- A blocked transition (invalid jump or failed guard) prints the failures and exits 1; `--force` overrides both.
- When `track_history` is enabled, successful transitions append a dated `lifecycle_log` entry to the spec frontmatter.
- `auto-promote` advances only specs whose next transition passes guards (or, with `--dry-run`, reports what would change without writing).
- `enforce` exits non-zero when any selected check (`require_status`, `check_allowed`, `check_max_age`) finds a violation; `status`, `history`, and `guard` honor JSON output.

## Out of Scope

- Defining the lifecycle graph itself (statuses and allowed transitions live in `types::SpecStatus`).
- Editing spec body content or any field other than `status:` / `lifecycle_log` in frontmatter.
- Interactive prompts and any GUI/web interface.

## Constraints

- Must not panic on expected error conditions — return Results or print and exit
- Must work with the project's Clap-based CLI argument parsing
- Lifecycle/guard configuration (guards, `track_history`, `allowed_statuses`, `max_age`) comes from the `[lifecycle]` section of `.specsync/config.toml` (v4 layout)
- Status mutations rewrite only the `status:` line inside YAML frontmatter delimiters — never the spec body
- `lifecycle_log` history may live in frontmatter or, post-`migrate`, in `.specsync/lifecycle/{module}.json`; both are read by `history`/`enforce`

### REQ-cmd-lifecycle-001

The `cmd_lifecycle` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.


### REQ-cmd-lifecycle-002

The `no_stale` guard SHALL NOT pass when staleness is unverifiable.

Acceptance Criteria
- A promotion gated on absence of staleness is blocked when git cannot answer, and the blocker names the reason.

### REQ-cmd-lifecycle-003

The lifecycle minimum-score gate SHALL remain inclusive and SHALL refuse a directory mapping on the same basis as `check`.

Acceptance Criteria
- A total equal to the configured minimum passes; only a total below it fails.
- A spec whose `files:` entry is a directory scores zero and therefore fails any positive minimum, matching the hard failure `check` already produces for the same mapping.

### REQ-cmd-lifecycle-004

The no-stale guard SHALL fail when a spec cites a source file that no longer exists.

Acceptance Criteria
- A deleted cited file fails the guard regardless of the configured threshold, since it measures a single commit and would otherwise pass on any threshold above one.
- The failure names the file and states that it is still cited by the spec.

### REQ-cmd-lifecycle-005

The `lifecycle` commands SHALL read spec files (8 MiB) and `.specsync/lifecycle/*.json` (4 MiB) only through the bounded project read of `util` (REQ-util-003), with a bound of the one named for each file. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list

### REQ-cmd-lifecycle-006

`lifecycle` SHALL write project files only through the confined project write functions of `util` (REQ-util-002), or through a write whose path was refused as a link just before; any other direct write SHALL say, on the write or the line above it in the same statement, why its path is not a project file (a temporary or Git directory, a test barrier, a file Git hands the command). It SHALL write a spec's new status through the confined project write.

Acceptance Criteria
- `tests/integration/project_writes.rs` `no_project_file_is_written_outside_the_confined_helpers` finds no unmarked direct write in this module's non-test source

