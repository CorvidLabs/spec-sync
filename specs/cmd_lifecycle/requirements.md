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

The lifecycle command SHALL enforce valid spec maturity transitions and SHALL preserve deterministic text and structured output.

Acceptance Criteria
- `promote`/`demote` use `SpecStatus::next()`/`prev()`; `set` accepts any valid status; all validate via `can_transition_to()` unless `--force`.
- Guard evaluation checks `min_score`, `require_sections`, and staleness (`no_stale`/`stale_threshold`), matching both specific (`from→to`) and wildcard (`*→to`) keys in either Unicode (`→`) or ASCII (`->`) form.
- A blocked transition (invalid jump or failed guard) prints the failures and exits 1; `--force` overrides both.
- When `track_history` is enabled, successful transitions append a dated `lifecycle_log` entry to the spec frontmatter.
- `auto-promote` advances only specs whose next transition passes guards (or, with `--dry-run`, reports what would change without writing).
- `enforce` exits non-zero when any selected check (`require_status`, `check_allowed`, `check_max_age`) finds a violation; `status`, `history`, and `guard` honor JSON output.

