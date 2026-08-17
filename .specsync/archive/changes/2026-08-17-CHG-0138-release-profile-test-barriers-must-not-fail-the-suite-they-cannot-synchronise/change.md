---
id: CHG-0138-release-profile-test-barriers-must-not-fail-the-suite-they-cannot-synchronise
state: archived
type: bug_fix
base_commit: fad39674c308e3d5c25a4324e838db69b14adc9c
---

# Release-profile test barriers must not fail the suite they cannot synchronise

## Intent

release-profile test barriers must not fail the suite they cannot synchronise

## Affected Canonical Specs

- None

## Acceptance Criteria

- cargo test --release exits 0 with the six affected tests reported as ignored rather than absent, and cargo test in debug still exits 0 with all of them running; the counts reconcile exactly (368 run + 6 ignored = 374 in release, 374 + 0 ignored in debug), so the gating hit the intended set and nothing else; no assertion is weakened, removed, renamed or deleted; the shipped release binary is byte-identical, proven by git diff touching no src/ file; the guards remain unconditional and demonstrably fire in a release binary under a live symlink race; comments state honestly which guards lose release-runnable coverage rather than citing a substitute test that covers a different guard.

## No-spec Rationale

Test gating only. No module's public contract or spec text changes; the shipped binary is byte-identical and its guards are unconditional. tests/ is not owned by any spec module.
