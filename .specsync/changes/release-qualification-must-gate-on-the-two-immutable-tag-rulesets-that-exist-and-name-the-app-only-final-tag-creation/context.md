---
change: release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation
artifact: context
---

# Context

`release.yml` has failed on every RC tag since the ruleset check landed in #492 (2026-08-01):
`v6.0.0-rc.1` through `rc.6`, six consecutive candidates. `v5.2.0` (2026-07-19) shipped only
because the check did not exist yet. The workflow has **never** passed with this check in place.

The `resolve` job demanded three repository tag rulesets plus `vars.SPECSYNC_RELEASE_APP_ID`:

| Ruleset | Needs an App? | Exists? |
|---------|---------------|---------|
| `SpecSync final tag creation` — blocks creation of `refs/tags/v*.*.*` (excluding rc) except for one bypass actor, the release App | yes | **no** |
| `SpecSync immutable final tags` — update+deletion, validated with `release_app_id=None` | no | yes (id 21432148) |
| `SpecSync immutable RC tags` — update+deletion on `refs/tags/v*.*.*-rc.*` | no | yes (id 21432132) |

Then it demanded a protected `release` deployment environment.

Verified against the live repository on 2026-08-25:

- `gh api repos/CorvidLabs/spec-sync/rulesets` returns the two immutability rulesets, both
  `target=tag`, `source_type=Repository`, `enforcement=active`, `bypass_actors=[]`, and no
  `SpecSync final tag creation` ruleset.
- `gh api repos/CorvidLabs/spec-sync/actions/variables` → `total_count: 0`. There is no
  `SPECSYNC_RELEASE_APP_ID`.
- `gh api repos/CorvidLabs/spec-sync/actions/secrets` → `total_count: 0`. There is no
  `SPECSYNC_RELEASE_APP_PRIVATE_KEY`.
- `gh api repos/CorvidLabs/spec-sync/environments` returns `github-pages` only. There is no
  `release` environment, so the `environment` check would 404 under `set -euo pipefail` even if
  the ruleset check were fixed.

So the release App was never provisioned. `--release-app-id "$RELEASE_APP_ID"` expanded to
`--release-app-id ""`, which argparse rejects before any ruleset is read — the two rulesets that
DO exist were never validated even once. That is the trap this repo exists to catch: a gate that
always fails proves less than no gate at all, because it hides the checks behind it.

## Decision taken

The repository owner decided to adopt the two App-free rulesets and skip the App-only creation
policy. The two rulesets were created and are live and active before this change was written.

The constraint this change is built around: **dropping a protection is allowed; dropping it
quietly is not.** So the validator refuses to succeed without emitting a non-empty `unenforced`
list, and `release.yml` prints every entry as a `::warning::` annotation and into the step summary
on every run. A green release cannot be read as evidence that App-only final-tag creation is
enforced, because the run itself says it is not.

## Already ruled out

- **Creating the App / the `release` environment.** Out of scope and not the owner's decision.
  `promote` keeps `vars.SPECSYNC_RELEASE_APP_ID` — the App is how promotion *pushes* a tag, not a
  policy stopping anyone else from creating one. Promotion stays unprovisioned and fails closed.
- **Switching `promote` to push with `GITHUB_TOKEN` + `contents: write`.** That would grant the
  default workflow token tag-write on every promote run. A larger security decision than the one
  delegated, and not needed to unblock RC qualification.
- **Relaxing the two surviving rulesets.** The validator deliberately refuses supersets, and both
  live payloads already pass its existing strict checks unmodified — confirmed before editing
  anything. No strictness was traded away to make this pass.
