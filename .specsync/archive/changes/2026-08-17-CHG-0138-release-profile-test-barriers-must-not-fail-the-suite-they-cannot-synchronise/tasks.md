---
change: CHG-0138-release-profile-test-barriers-must-not-fail-the-suite-they-cannot-synchronise
artifact: tasks
---

# Tasks

- [x] Measure the real failure list rather than trusting the issue's count. Five surfaced; a sixth
      was already `#[cfg(all(unix, debug_assertions))]` and a seventh and eighth are Windows-only
      with the identical defect.
- [x] Inventory EVERY `#[cfg(debug_assertions)]` and `debug_assert*` in `src/` — not only the ones
      the failing tests touch — and classify each as rendezvous or guard.
- [x] Verify each of the four named guards carries no `cfg` attribute and every call site is
      unconditional.
- [x] Confirm the three `#[cfg(not(debug_assertions))]` arms are barrier stubs, each immediately
      followed by an unconditional guard.
- [x] Prove empirically that the release binary still refuses under a live symlink race, rather
      than reasoning about it from source.
- [x] Choose per site between "the guard should run in release" and "the test cannot run in
      release", and justify the choice concretely rather than by preference.
- [x] Use `cfg_attr(not(debug_assertions), ignore)` rather than `cfg`, so the tests stay compiled,
      type-checked and VISIBLE in run output instead of silently absent.
- [x] Convert the pre-existing `#[cfg(all(unix, debug_assertions))]` test to the same form — under
      `cfg` it was not even type-checked in release and could rot unnoticed.
- [x] Prove both profiles: `cargo test --release` and `cargo test` both exit 0, and the ignored
      count reconciles exactly against the intended set.
- [x] Confirm no assertion was weakened, removed, renamed or deleted.
- [x] Submit to adversarial verification before merging, given the harness flagged it as a
      possible security-test removal.
- [x] Correct the false substitute-coverage claim: name the guards that genuinely have no
      release-runnable test instead of citing a test that asserts a different guard.
- [x] Correct the CI claim — `release.yml:633` runs qualify on `windows-latest`, so the two
      Windows tests do run there.
- [x] Credit the release-runnable test that WAS found for `open_server_root_capability`.
- [x] File the residual release-coverage gap separately (#614) rather than papering over it.
- [x] CHANGELOG entry stating the lost release coverage plainly.
