---
id: CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped
state: implementing
type: bug_fix
base_commit: 4c936cd6f4a9dae7e138a90d3af1709c8ee6e2f4
---

# Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths)

## Intent

Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths)

## Affected Canonical Specs

- `changelog`
- `cli`
- `cli_args`
- `cmd_changelog`
- `cmd_deps`
- `cmd_diff`
- `cmd_init`
- `cmd_init_registry`
- `cmd_lifecycle`
- `cmd_report`
- `cmd_resolve`
- `cmd_scaffold`
- `cmd_score`
- `cmd_stale`
- `cmd_wizard`
- `commands`
- `comment`
- `config`
- `deps`
- `generator`
- `git_utils`
- `hash_cache`
- `hooks`
- `ignore`
- `importer`
- `output`
- `parser`
- `registry`
- `scoring`
- `types`
- `util`
- `validator`
- `agents`
- `cmd_check`
- `cmd_coverage`

## Acceptance Criteria

- Pre-6.0 product fixes land on main: hooks managed-block safety, init honesty, draft-planned coverage correctness, module naming, exit-code/CI helpers; cargo test and clippy -D warnings green.

## No-spec Rationale

Not applicable
