---
change: CHG-0074-simplify-specsync-ci-to-one-expensive-suite-authority-with-residual-trust-identi
artifact: testing
---

# Testing

## Requirement Evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-006` | `fledge lanes validate`, the one-step `trust-lifecycle` run, thin-diff inspection, and recorded `change check` evidence for both validators, 2,178 unit tests, 333 integration tests, and release validation prove single hosted suite ownership while retaining full product confidence. |

## Focused checks

- `fledge lanes run trust-lifecycle --non-interactive` completes quickly and runs `check-types`
  without invoking the `test`, `lint`, or `verify` lanes.
- Parse `fledge.toml` and `.trust.toml` to assert that Trust targets `trust-lifecycle`, the residual
  lane excludes expensive tasks, and `verify` still contains the full suite.
- `git diff --name-only origin/main...HEAD` contains no `.github/workflows/**`, protected script,
  `src/**`, or `specs/cmd_change/**` path.

## Completion checks

- [x] `fledge lanes validate --non-interactive`
- [x] `fledge lanes run trust-lifecycle --non-interactive`
- [x] `specsync change check <id>` (includes the one full `cargo test` run)
- [ ] Run residual Trust against the current candidate binary after the implementation commit
- [ ] Confirm the hosted PR's strict 100% spec/path coverage gate

`fledge lanes run verify` remains the full local suite by configuration. It is intentionally not
run after `change check`, because both select the same full `cargo test` suite and this change exists
to remove that kind of duplicate execution.

## Hosted observation

Compare the prior Trust job (roughly 20 minutes, roughly 17 minutes in duplicated tests) with the
new Trust lifecycle timing. Product PR wall clock may still be bounded by the current protected
macOS/Windows/coverage jobs until the separately pinned Tier B workflow change lands.

Focused local observation: `trust-lifecycle` selected exactly one `check-types` step and completed
in 18.1 seconds from a cold target; `fledge lanes validate` reported seven valid lanes.
