---
change: release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation
artifact: research
---

# Research

Everything below was measured against the live repository on 2026-08-25, not assumed.

## Live rulesets

`gh api repos/CorvidLabs/spec-sync/rulesets` returns three entries: `Copilot`
(`target=branch`, `enforcement=disabled`) and the two tag rulesets.

| id | name | target | source_type | enforcement | rules | include | exclude | bypass_actors |
|----|------|--------|-------------|-------------|-------|---------|---------|---------------|
| 21432132 | `SpecSync immutable RC tags` | tag | Repository | active | `update`, `deletion` | `refs/tags/v*.*.*-rc.*` | — | `[]` |
| 21432148 | `SpecSync immutable final tags` | tag | Repository | active | `update`, `deletion` | `refs/tags/v*.*.*` | `refs/tags/v*.*.*-rc.*` | `[]` |

No `SpecSync final tag creation` ruleset exists.

Both payloads carry exactly the field set the validator already allows: required `name`, `target`,
`source_type`, `enforcement`, `bypass_actors`, `conditions`, `rules`, plus optional `id`,
`node_id`, `source`, `current_user_can_bypass`, `_links`, `created_at`, `updated_at`. No unknown
key, so `require_exact_object_fields` passes untouched.

## Why the job died

`vars.SPECSYNC_RELEASE_APP_ID` does not exist — `gh api repos/CorvidLabs/spec-sync/actions/variables`
returns `total_count: 0`. In the workflow it expanded to the empty string, so the command ran as
`--release-app-id ""`, and `argparse` with `type=int` rejects that before a single ruleset file is
read. The failure was therefore never about the rulesets at all; it happened before they were
examined. `secrets` is likewise empty (`total_count: 0`), so
`SPECSYNC_RELEASE_APP_PRIVATE_KEY` is absent too.

Even with that fixed, `resolve_ruleset "SpecSync final tag creation"` would have found zero
matching ids and exited 1 with `Repository requires exactly one … ruleset`.

Even past that, `gh api repos/.../environments/release` 404s: `gh api
repos/CorvidLabs/spec-sync/environments` lists only `github-pages`. Under `set -euo pipefail` that
fails the step. So three independent blockers stood between an RC tag and a green `resolve`.

## Pre-change strictness baseline

Before editing the validator, both live payloads were passed to the **unmodified**
`validate_rc_tag_ruleset` and `validate_final_tag_immutability_ruleset`. Both returned
`valid: true`. This is the load-bearing fact for "no silent weakening": the live configuration was
already acceptable to the strict validators, so nothing in this change loosens a check in order to
make reality fit. What changed is which checks are demanded, not how strict the surviving ones are
— and the bypass rule actually tightened, since `validate_tag_ruleset` no longer has a parameter
that admits a bypass actor at all.

## Blast radius

`grep` for `SPECSYNC_RELEASE_APP_ID` / `release_app_id` across the repo (excluding `target/`,
`.git/`, and `.specsync/archive/`) found: the `resolve` env binding and the `promote` token step in
`release.yml`, the validator's parameter and CLI flag, the test suite, and archived change records.
Archived changes are history and are not rewritten. `deployments: read` appears exactly once in
`.github/workflows/`, at the top of `release.yml`, and its only consumer was the environments
query.
