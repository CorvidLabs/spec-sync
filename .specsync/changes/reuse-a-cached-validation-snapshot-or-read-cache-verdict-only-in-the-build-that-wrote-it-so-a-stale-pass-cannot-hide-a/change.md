---
id: reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a
state: implementing
type: bug_fix
base_commit: cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b
---

# Reuse a cached validation snapshot or read-cache verdict only in the build that wrote it, so a stale pass cannot hide a new refusal (#878)

## Intent

Reuse a cached validation snapshot or read-cache verdict only in the build that wrote it, so a stale pass cannot hide a new refusal (#878)

## Affected Canonical Specs

- `hash_cache`
- `change`

## Acceptance Criteria

- A validation snapshot recorded by any other build, including one that reports the same package version and one written before #878, is not replayed: a default specsync check re-validates the spec and reports the refusal this build makes (a duplicated requirement ID in the #878 case), while a snapshot recorded by this build still replays.
- The persistent read cache written by another build of the same version, such as a sibling worktree sharing the Git common directory, is not loaded; its passing verdicts are recomputed live.
- Both bindings come from one build fingerprint compiled in by build.rs over src/, Cargo.toml, Cargo.lock and build.rs; dropping it from either digest fails a test (mutation), and the end-to-end repro (released 6.2.0 then this build) reports the duplicate instead of All specs unchanged.

## No-spec Rationale

Not applicable
