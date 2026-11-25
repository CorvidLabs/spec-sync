---
change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
artifact: design
---

# Design

## Parts

| Part | Lane | What runs |
|---|---|---|
| `tests-1-of-4` to `tests-4-of-4` | `release-candidate-tests` | `cargo nextest run --no-fail-fast --partition slice:K/4` |
| `build` | `release-candidate-build` | `test-release-guards`, then `build` |

`fledge lanes run release-candidate` stays the whole lane for local use: the unpartitioned tests, the release guards and the build.

## Part record

One JSON file per part, uploaded as `rc-part-<platform>-<part>`. Its fields are `schema_version` 1, `platform`, `part`, `lane`, `outcome`, `rc_tag`, `candidate_sha`, `package_version` and `workflow_revision`, and for a test slice `tests_in_part` and `tests_listed`. The counts come from `cargo nextest list --message-format json`, with and without the partition, counting the tests that match the run's filter. A part that fails still writes its record with the job's outcome, as the platform record does today.

## Platform evidence

`record-qualification` downloads the part records and runs `validate-release-candidate.py parts`. For each required platform it requires exactly the five named parts, each successful and bound to the candidate tag, SHA, package version and one workflow revision, the same `tests_listed` in every slice, and slice counts that add up to it. It then writes `<platform>.json` in the existing schema with `lane: release-candidate` and the outcome, and the job uploads them as `rc-evidence-<platform>`. The existing `evidence` validation runs on those files, and `authorize-release` and `release` download and validate them unchanged.

A platform whose parts do not pass the check gets a record with outcome `failure`, so the existing validation refuses the candidate and the gate is published as failed.

## Rehearsal

`rc-rehearsal.yml` holds the same matrix, pins, commands and limit, with no evidence and no gate. It starts on `workflow_dispatch`, once a week, and on a pull request that changes `release.yml`, `rc-rehearsal.yml`, `fledge.toml` or the part script. A contract test compares it with `release.yml`.

## Time limits

`.github/scripts/test-workflow-time-limits.py` parses every workflow and fails on a job with no `timeout-minutes` or one above 30. The limits that change: `qualify` 120 to 30; in `ci.yml` `test` 45, `closed-late-compat` 40 and `linux-baseline` 45 to 30; the `build` job of `release.yml` and of `rc-assets.yml` 45 to 30; and `trust` and `attest`, which had none.
