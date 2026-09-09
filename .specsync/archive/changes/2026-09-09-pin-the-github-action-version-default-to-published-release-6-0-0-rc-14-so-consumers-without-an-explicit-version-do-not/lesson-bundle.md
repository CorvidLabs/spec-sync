# Lesson bundle — pin-the-github-action-version-default-to-published-release-6-0-0-rc-14-so-consumers-without-an-explicit-version-do-not

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Pin the GitHub Action version default to published release 6.0.0-rc.14 so consumers without an explicit version do not 404 on missing v6.0.0 (#628)
- **Kind**: BugFix
- **Specs**: github
- **Paths**: action.yml, site/src/content/docs/integrations/github-action.md, .github/scripts/validate-release-version.py, specs/github/requirements.md, specs/github/github.spec.md, specs/github/testing.md, specs/github/context.md, specs/github/tasks.md
- **Acceptance**: action.yml version default is 6.0.0-rc.14 (no v prefix); description names a published release candidate and does not imply stable 6.0.0 exists; site GitHub Action inputs table default matches; validate-release-version.py accepts Action/docs default 6.0.0-rc.14 during the pre-stable window while Cargo.toml remains 6.0.0; README immutable pin examples may stay at package version; python3 -S .github/scripts/validate-release-version.py passes; gh release view v6.0.0-rc.14 shows assets; specs/github REQ-github-002 covers the candidate-window pin to a published release with assets

## Evidence

- Verification commit: `5b275087eea37094b7495893d034415f5ba650ee`
- Base commit: `d0da6075f8f383c474915dd8d2b97dfd9e8b776f`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

Issue #628: `action.yml` defaults `version` to `6.0.0`, but no GitHub Release `v6.0.0` exists.
Any consumer that omits the optional `version:` input downloads a 404. Verified live:

- `gh release view v6.0.0` → release not found
- `gh release view v6.0.0-rc.14` → prerelease with the five Linux/macOS archives + sha256 sidecars
- Tags `v6.0.0-rc.15`..`rc.17` exist but have **no** Release assets — must not become the default

`validate-release-version.py` currently requires `action.yml` default and the site Action docs
default to equal `Cargo.toml`'s package version (`6.0.0`). That sync anticipated stable
publication and is exactly what made the broken default pass CI while consumers 404'd.

`REQ-github-002` says the composite Action's current default matches the promoted stable
package version. During the candidate window the package version is already `6.0.0` but has
not been published; the downloadable default must resolve to a Release that has assets.

Out of scope: changing README/`@v6.0.0` immutable pin *examples* (those document the intended
stable pin and stay synced to Cargo.toml), bumping Cargo.toml, publishing `v6.0.0`, or
defaulting to `rc.15+` / `latest` / `5.2.0`.

Succession note: delivery paths (`action.yml`, site docs, the validator script) stay
`@exact:delivery` without `[modules."github"] owns` — granting owns requires editing
`.specsync/config.toml`, which cannot itself be owned and whose reopen replayed a stale
Invariants delta. Spec companions under `specs/github/` are superseded from CHG-0082.

## From the change's design.md

# Design

## Decision

Pin the Action `version` **default** (and the site inputs-table default) to `6.0.0-rc.14`.
Keep Cargo.toml at `6.0.0`. Teach `validate-release-version.py` a narrow candidate-window
exception for those two surfaces only.

## Alternatives rejected

| Option | Why not |
|---|---|
| Default `5.2.0` (latest stable) | Parent scope for #628 chose the latest published **6.0** candidate with assets |
| Default `latest` | Moves under consumers; description already discourages it for CI |
| Default `6.0.0-rc.15`..`rc.17` | Tags exist; Releases have **no** assets |
| Change Cargo.toml to `6.0.0-rc.14` | Binaries self-report `6.0.0`; package version is not the download pin |
| Add `[modules."github"] owns` for `action.yml` | Requires editing `.specsync/config.toml`, which cannot be owned and reopen of CHG-0082 rematerialized a stale Invariants delta onto tip |

## Validator shape

Introduce `PUBLISHED_ACTION_DEFAULT = "6.0.0-rc.14"`. When package version is `6.0.0` and no
stable release has been published yet, `action.yml` default and the site docs default must
equal `PUBLISHED_ACTION_DEFAULT`. All other surfaces (README pins, CI consumer with mirror,
Trust pin) keep requiring package version equality.

## Spec delta

Amend `REQ-github-002` so the "current default matches promoted stable package version" rule
applies after stable publication; before that, the default must be a published Release with
assets for the version under promotion.

## From the change's testing.md

# Testing

## Automated

| Command | Covers |
|---|---|
| `python3 -S .github/scripts/validate-release-version.py` | Action default + docs default accept `6.0.0-rc.14`; README pins and CI/Trust mirrors still match package `6.0.0` |
| `specsync check --spec github` | Living github module still coherent after REQ-github-002 amendment |
| `specsync change check` | Materialize deltas + scoped verification for this change |

## Manual / dogfood

- `gh release view v6.0.0-rc.14` lists the five archives + sha256 sidecars
- `gh release view v6.0.0` → not found
- Default URL shape `.../releases/download/v6.0.0-rc.14/specsync-linux-x86_64.tar.gz` resolves
  (HTTP 200 or redirect to asset), while `.../v6.0.0/...` does not

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-github-002 | action.yml default `6.0.0-rc.14`; docs table matches; validator passes; live release view shows assets |

## Where these lessons go

- `specs/github/context.md`
