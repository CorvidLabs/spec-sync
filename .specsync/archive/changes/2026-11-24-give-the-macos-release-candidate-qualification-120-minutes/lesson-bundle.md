# Lesson bundle: give-the-macos-release-candidate-qualification-120-minutes

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Give the macOS release-candidate qualification 120 minutes
- **Kind**: Operations
- **Paths**: .github/workflows/release.yml, docs/releases/6.4.2.md
- **Acceptance**: The `qualify` job in `.github/workflows/release.yml` has `timeout-minutes: 120`, and its comment says why. The 6.4.2 handoff records that `v6.4.2-rc.1` was cancelled on macOS at 75 minutes with every test passed and that the candidate is cut again. The version validator, the workflow-pin validator and the release-candidate contract tests pass. No product, test or spec file changes.

## Evidence

- Verification commit: `ffca9ef0a6e5147522a7db883b1a56103d18d25e`
- Base commit: `b114d09fd1b102ddbb57dd4032769d6c854e5c52`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

`v6.4.2-rc.1` did not qualify. Ubuntu passed in 15m59s. On macOS all 4,176 tests passed, but that step took 69m41s and the `qualify` job was cancelled at its 75-minute limit while `test-release-guards` was compiling. A complete macOS run needs about 84 minutes, so a re-run cannot fit. `v6.2.0-rc.1` was cancelled the same way at 45 minutes, and #860 raised the limit to 75.

The whole increase since 6.4.1 is the 277 tests this release added. The time is Git process starts: the slowest test starts 11,576 of them, at about 18.5 ms each on the hosted macOS runner and 2.8 ms on Ubuntu. #1161 has the measurements.

0xLeif decided to raise the limit to 120 and cut `v6.4.2-rc.2`, so the release is not held. He also said he does not want the limit raised again and wants every test lane to finish within about 30 minutes. That work is #1161. This change moves away from SPEED-2.b.1 on purpose and for now. SPEED-2.b.2 holds: no test is dropped or weakened.

Ruled out for today: re-running rc.1, which would time out again, and splitting the macOS run into parallel jobs, which changes the candidate evidence records and their validator and is not a change to make while a release waits.

## From the change's testing.md

# Testing

Run `python3 .github/scripts/validate-release-version.py`, `python3 .github/scripts/validate-workflow-runtime-pins.py`, `python3 .github/scripts/test-validate-release-candidate.py` and the required-gate tests on the edited tree. Parse `.github/workflows/release.yml` as YAML and read `jobs.qualify.timeout-minutes` back as 120, with every other job's limit unchanged.

The proof that 120 is enough is the `v6.4.2-rc.2` qualification itself, on Ubuntu and macOS.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
