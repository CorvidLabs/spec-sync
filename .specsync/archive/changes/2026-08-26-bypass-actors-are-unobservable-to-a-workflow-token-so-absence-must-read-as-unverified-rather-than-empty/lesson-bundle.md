# Lesson bundle — bypass-actors-are-unobservable-to-a-workflow-token-so-absence-must-read-as-unverified-rather-than-empty

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Bypass actors are unobservable to a workflow token, so absence must read as unverified rather than empty
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/scripts/validate-release-candidate.py, .github/scripts/test-validate-release-candidate.py, specs/github/github.spec.md
- **Acceptance**: an absent bypass_actors field validates as unverified and names what was not checked, rather than failing the gate
- **Acceptance**: a visible bypass actor is still refused
- **Acceptance**: an admin payload with no bypass actors passes with no notice

## Evidence

- Verification commit: `d508f144a1d965b395abfe45f23c8b4e8978cd5f`
- Base commit: `d508f144a1d965b395abfe45f23c8b4e8978cd5f`
- Verified by: `bash .github/scripts/test-classify-ci-paths.sh`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`

## From the change's context.md

# Context

`release.yml` failed on the rc.7 tag with:

    error: final tag immutability ruleset is missing fields: bypass_actors

GitHub returns `bypass_actors` only to a caller with **admin access to repository settings**. The
workflow runs with `contents: read, actions: read, checks: read`, so the field is absent from every
payload it fetches. The validator listed it in `REQUIRED_RULESET_FIELDS` and refused.

**This gate could never have passed from CI.** Three earlier failures hid it: first no rulesets
existed at all, then `--release-app-id ""` was rejected by argparse before a single ruleset file was
read. Each fix revealed the next layer.

It was also invisible locally, because a maintainer's `gh` is authenticated as an admin and does see
the field. The only way to observe it was to read what the runner's token can see, or to watch the
lane fail again.

## The shape

Absence and emptiness are different, and reading one as the other is the defect this release has
fixed repeatedly (#672, #684, #689's first design, #704's compatibility path). The validator
enforcing that distinction elsewhere had it backwards here: it treated an unobservable field as a
malformed payload.

Absence now means UNOBSERVED. It is validated when visible, and when not, it joins the enforced
disclosure list — the workflow already fails if that list is empty, so the admission cannot be
quietly dropped.

## From the change's testing.md

# Testing

Three payloads, because the risk is trading one silent hole for another:

- `test_absent_bypass_actors_reads_as_unverified_not_as_empty` — the regression. The field is
  deleted from both fixtures, mimicking what a non-admin token receives. Validation passes and
  emits exactly two notices naming the rulesets that were not checked.
- `test_a_visible_bypass_actor_is_still_refused` — **honest label: the CONTROL.** Softening absence
  must not soften presence. This is what fails if the check were relaxed into accepting any value.
- The existing accept test covers the third case: an admin payload with no bypass actors passes
  and emits no such notice.

Also run against the **live rulesets** rather than fixtures only: the real payloads pass, the same
payloads with `bypass_actors` stripped pass with notices, and a payload granting bypass is refused.

50 validator tests pass.

## Where these lessons go

- `specs/github/context.md`
