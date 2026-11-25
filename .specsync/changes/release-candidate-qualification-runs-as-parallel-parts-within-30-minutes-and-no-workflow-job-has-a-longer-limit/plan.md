---
change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
artifact: plan
---

# Plan

## Approach

Split the `qualify` job of `release.yml` into a matrix of platform by part: four test slices and one release-build part on each of Ubuntu and macOS, each with a 30-minute limit. Each part uploads a small record. `record-qualification` checks the ten records with the validator and writes the two platform evidence files in their existing form, so promotion and publication read what they read today. A rehearsal workflow runs the same parts without a candidate. A contract test holds every workflow job to a limit of at most 30 minutes.

## Out of scope

- Starting fewer Git processes for each lifecycle operation (the second half of #1161): product code, its own change.
- Any change to a test, to what a test asserts, or to which tests exist.
- The form of the platform evidence that `authorize-release`, `promote` and `release` validate.
- Windows, which is neither built nor qualified.

## Steps

1. Contract tests first, failing: the validator's `parts` mode, the workflow time-limit test, the rehearsal lockstep test, and the pins on the `qualify` matrix.
2. `fledge.toml`: lanes `release-candidate-tests` and `release-candidate-build`, with the whole `release-candidate` lane kept for local use.
3. `.github/scripts/release-candidate-part.sh`: run one part and write its record, with the slice counts from `cargo nextest list`.
4. `validate-release-candidate.py parts`: check the part records and write the platform evidence.
5. `release.yml`: the part matrix, the 30-minute limit, the part artifacts, and `record-qualification` building the platform evidence from them.
6. `rc-rehearsal.yml`, and the limits in `ci.yml`, `rc-assets.yml`, `trust.yml` and `release.yml`.
7. `docs/RELEASING.md`, `docs/ci-confidence.md`, `AGENTS.md`, the changelog and the github spec's testing notes.
8. Rehearse on the pull request and record each part's time.

## Risks

- The first hosted run of the split could be slower than the estimate. The rehearsal on this pull request measures it before merge, and the next candidate is cut only after a rehearsal passes.
- `record-qualification` now writes the platform evidence, where each platform's job wrote its own. A mistake there could call a failed platform successful. The validator's `parts` mode is the only writer, it is tested against every refusal, and the existing evidence validation still runs on what it wrote.
- A candidate qualified before this change and promoted after it would meet a newer workflow. None is in flight: 6.4.2 is published.
- Ten jobs instead of two start cold, since the release lane restores caches and never saves them. That costs runner minutes, which are free on a public repository.

## Constraints consulted

- SPEED-2.b.2: no test is dropped or weakened. The slices are cargo-nextest's own partition of the unchanged list, and the validator refuses slices that do not add up.
- REQ-github-007: one immutable candidate, both platforms green, release-chain pins unchanged, platform evidence revalidated at publication.
- The release lane restores caches and never saves them; that stays.
- CI-6 and the required CI gate: `ci.yml` keeps no path filter, and the rehearsal is a separate workflow that no required check depends on.
