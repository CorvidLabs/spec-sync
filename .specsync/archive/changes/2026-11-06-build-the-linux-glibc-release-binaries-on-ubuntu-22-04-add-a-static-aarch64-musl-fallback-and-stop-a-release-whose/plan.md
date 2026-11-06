---
change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
artifact: plan
---

# Plan

## Approach

Link the two Linux glibc downloads on the documented floor: GitHub's Ubuntu 22.04 runners, built
natively. Add a static ARM64 musl download as the below-floor fallback. Make one check script,
which runs the packaged, checksum-verified archive on the baseline images, the gate in both
release workflows, and exercise the same recipe and check in PR CI. Write the failing test first.

## Out of scope

- Publishing, tagging, dispatching `release.yml` or `rc-assets.yml`, or changing any release.
- An Action-side glibc probe or automatic musl selection. GitHub-hosted Ubuntu images are all at
  or above the floor after this change; the loader's own message is documented with its recovery.
- The Homebrew tap formula, which lives in a separate repository; RELEASING.md notes the new asset.
- Windows, macOS builds, and the musl build runner for x86_64 (static, so unchanged).
- The x86_64 Action-consumer CI job, which builds on `ubuntu-latest` and runs there.

## Steps

1. **Failing test first.** Write `linux-baseline.py` and `test-linux-baseline.py`. Then run them
   against the current state and record the red results as Chough evidence:
   - the check rejects the published v6.2.0 aarch64 archive on `ubuntu:22.04` and `debian:12-slim`
     with `GLIBC_2.39`;
   - the structural test fails on the current workflows (`ubuntu-latest` gnu rows, no aarch64
     musl, no check step, no CI job).
2. Change the `release.yml` and `rc-assets.yml` build matrices: gnu rows on Ubuntu 22.04
   runners, the new aarch64 musl row, the cross linker removed, per-target musl env, the check
   step before upload, and comments.
3. Add the `ci.yml` `linux-baseline` job with the negative control. Wire it into
   `implementation-gate` and `corvid-pet`, and add the path filter. In
   `test-required-ci-gate.py`, add `linux-baseline` to the expected-skipped sets of the
   archive-only, review-only and site-only scenarios; no expectation is removed. This file was
   added to scope by Leif's "Yes, widen" send-back.
4. Add the aarch64 musl archive to `EXPECTED_ARTIFACT_ARCHIVES` and update its tests.
5. Add the `linux-baseline-test` fledge task to the `verify`, `ci` and `repo` lanes.
6. Docs: README, Action docs, RELEASING.md, HLD.md, SCOPE.md, CHANGELOG. `specsync change check` materializes REQ-github-025; then bump the `github` spec version, add its Change Log row and a `github.3md` lesson plane.
7. Green: build the ARM64 glibc and musl archives locally with the new recipe (in Ubuntu 22.04
   and 24.04 containers), then run the check on them. Run the structural, release-validator and
   CI-gate tests, then `fledge lanes run verify`, `specsync check --strict`, `hi check` and the
   pre-push gate. Record evidence.
8. Run `specsync change check --commit`, then hand off as `ready for review`.

Estimate, by relative size, because `chough timing` history for this kind of change is thin:
- medium: script and tests;
- small: workflows and docs;
- the main uncertainty is the first hosted run on `ubuntu-22.04-arm`, which CI only performs on
  the archived tip.

## Risks

- **First hosted run.** `ubuntu-22.04-arm` and `ubuntu-24.04-arm` behavior (Docker, rustup,
  `musl-tools`, Python 3.10) is proven only when CI runs on the archived tip. Mitigations: the
  script targets Python 3.10, and local container builds reproduce the same OS and toolchain. Any
  hosted failure is fixed on top within scope.
- **Runner retirement.** When GitHub retires its Ubuntu 22.04 runner images, the release build
  stops at job start. That is loud, never a silent floor raise. The recovery is stated in the
  workflow comment and in RELEASING.md.
- **Docker Hub availability.** Pulls retry three times. A persistent outage fails the release
  build, which is the safe side.
- **Longer CI.** CI time grows by four parallel release builds on full runs.

## Constraints consulted

- `specs/github/github.3md`: hermetic release guards (stdlib-only Python run with `-I`, YAML read through Ruby's Psych); archive-first CI; offline tests do not prove publication or platform qualification. The release-matrix contract has lived in the `github` spec since CHG-0075 and the Windows drop, so this change records REQ-github-025 there rather than shipping as no-spec, as CHG-0048 did.
- CHG-0048 (#392) and the musl toolchain pattern.
- `release.yml` "NO CACHE HERE" on the publishing job (kept: no cache is added there).
- The `implementation-gate` rules enforced by `test-required-ci-gate.py` (#796).
- Archive-first CI.
- AGENTS rules: never weaken a test, gate or timeout; a refusal names a recovery that a test runs.
  The check's rebuild recovery is exactly what the `linux-baseline` CI job runs.
