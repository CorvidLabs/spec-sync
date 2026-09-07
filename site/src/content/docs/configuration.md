---
title: "Configuration"
section: "Reference"
order: 1
---

SpecSync 6.0 separates canonical validation settings from the verified SDD policy:

- `.specsync/config.toml` configures specs, source discovery, rules, maturity guards, and integrations.
- `.specsync/sdd.json` enables and protects the change lifecycle, meaningful paths, verification commands, and custom artifacts.

Neither file accepts provider credentials or model commands.

## Initialize

```bash
specsync init
```

New projects receive the current layout. Existing projects can migrate configuration with
`specsync migrate` and explicitly adopt the current verified lifecycle with
`specsync change adopt`. Adoption sets `enabled: true` on a policy written off and otherwise
preserves existing policy and lifecycle evidence byte-for-byte, records the pre-v2 cutoff, and
routes subsequent changes through the 6.0 single workflow. Existing
v1 changes must already be present at the trusted cutoff; adoption is atomic and fails without
changing the project when that compatibility condition is not met.

Configuration resolution is:

```text
.specsync/config.toml
.specsync/config.json      legacy fallback
.specsync.toml             legacy fallback
specsync.json              legacy fallback
defaults
```

## Canonical TOML

```toml
specs_dir = "specs"
source_dirs = ["src"]
schema_dir = "db/migrations"
schema_pattern = "CREATE (?:VIRTUAL )?TABLE(?:\\s+IF NOT EXISTS)?\\s+(\\w+)"
exclude_dirs = ["__tests__"]
exclude_patterns = ["**/__tests__/**", "**/*.test.ts"]
source_extensions = ["rs", "ts"]
include_extensionless = true
required_sections = ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"]
export_level = "member"
parse_mode = "ast"
enforcement = "strict"
task_archive_days = 30

[rules]
max_changelog_entries = 20
require_behavioral_examples = true
min_invariants = 1
max_spec_size_kb = 50
require_depends_on = false

[github]
repo = "owner/repo"
drift_labels = ["spec-drift"]
verify_issues = true

[companions]
design = true
```

All keys use `snake_case`. `specsync migrate` preserves explicit empty arrays and refuses lossy conversion when legacy custom rules cannot be represented safely.

## Core options

| Option | Type | Default | Purpose |
|---|---|---|---|
| `specs_dir` | string | `"specs"` | Recursive canonical spec directory |
| `source_dirs` | string[] | auto-detected or `src` | Source roots used for coverage and generation |
| `schema_dir` | string | unset | SQL migration directory |
| `schema_pattern` | string | built-in CREATE TABLE pattern | Table-name capture regex |
| `required_sections` | string[] | seven contract sections | Required `##` headings |
| `exclude_dirs` | string[] | common test directories | Directory exclusions |
| `exclude_patterns` | string[] | common test globs | Additive source exclusions |
| `source_extensions` | string[] | all supported | Restrict source discovery |
| `include_extensionless` | boolean | `false` | Add files without a filename extension to source discovery |
| `export_level` | `member` or `type` | `member` | Public-symbol validation depth |
| `parse_mode` | `regex` or `ast` | `regex` | Extraction strategy where AST support exists |
| `enforcement` | `warn`, `enforce-new`, or `strict` | `strict` | Default failure policy. `strict` exits 1 on any validation error; `warn` reports and exits 0 |
| `task_archive_days` | integer | unset | Age threshold used by task archival |

An omitted or empty `source_extensions` list continues to select all supported language extensions. Set `include_extensionless = true` to additionally discover files such as `bin/tool`; it is additive whether `source_extensions` uses the defaults or an explicit list.

## Validation rules

The `[rules]` table controls spec-quality constraints:

| Key | Purpose |
|---|---|
| `max_changelog_entries` | Warn when a Change Log needs compaction |
| `require_behavioral_examples` | Require a populated Behavioral Examples section |
| `min_invariants` | Set the minimum invariant count |
| `max_spec_size_kb` | Warn on oversized specs |
| `require_depends_on` | Require explicit dependencies |

## Maturity lifecycle guards

Module maturity (`draft`, `review`, `active`, `stable`, `deprecated`, `archived`) is separate from the SDD delivery lifecycle. Configure allowed states, age guidance, and transition guards in TOML:

```toml
[lifecycle]
track_history = true
allowed_statuses = ["draft", "review", "active", "stable", "deprecated", "archived"]

[lifecycle.max_age]
draft = 30
review = 14

[lifecycle.guards."review->active"]
min_score = 80
require_sections = ["Behavioral Examples", "Error Cases"]
no_stale = true
stale_threshold = 5
message = "Resolve drift before activation"
```

Use `specsync lifecycle`, `specsync stale`, and `specsync score` to inspect these policies. They do not replace the verified SDD states described below.

## Verified SDD policy

`.specsync/sdd.json` is versioned independently so upgrading cannot silently introduce new CI gates:

```json
{
  "version": 1,
  "enabled": true,
  "require_change_for_meaningful_files": true,
  "meaningful_paths": ["src/", "tests/", "site/", ".github/", "Cargo.toml"],
  "ignored_paths": [".specsync/", "specs/"],
  "verification_commands": ["fledge run test"],
  "custom_artifacts": {},
  "principles_file": null
}
```

`verification_commands` is retained on the policy file for adopters who still list them, but `change check` does not execute the list. Spec↔code sync is the verifier; CI owns the project's tests. Committed policy/configuration files remain meaningful even if a broad ignored path would otherwise cover them.

`custom_artifacts` maps an artifact name to a project-owned Markdown template. `principles_file` optionally adds project governance to interviews and agent context.

## Custom modules

Override automatic source ownership when a module spans unusual paths:

```toml
[modules."auth"]
files = ["src/auth/service.ts", "src/auth/middleware.ts"]
depends_on = ["database"]
owns = ["tests/auth", "Package.swift"]

[modules."api"]
files = ["src/routes/"]
depends_on = ["auth", "database"]
```

`owns` gives a module paths beyond its spec's `files:` for the change lifecycle — a file, or a directory that owns everything beneath it. An acceptance manifest signs an owned path under the module instead of the reserved `@exact:test` / `@exact:delivery` owners, so a later change can `change supersede` it under that module even when the change that first signed it is archived. Owned paths are not source mappings: `check` demands no spec coverage for them, and nothing under `.specsync/` can be owned.

## GitHub integration

```toml
[github]
repo = "CorvidLabs/spec-sync"
drift_labels = ["spec-drift", "needs-update"]
verify_issues = true
```

The repository is auto-detected from Git when `repo` is omitted.

## Agent-native enrichment

`specsync generate` is deterministic and local. Use `specsync agents install` or `specsync mcp` to let an existing coding agent enrich artifacts under that agent's own credentials and permissions. Legacy AI configuration keys are ignored with migration guidance; their values are never printed or executed.

See [AI and coding agents](integrations/ai-agents.md) and the [workflow guide](workflow.md) for the surrounding lifecycle.
