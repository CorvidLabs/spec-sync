---
change: reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a
artifact: research
---

# Research

- End-to-end reproduction (`repro-878.sh`, in a disposable fixture, run on 2026-11-06):
  1. Released specsync 6.2.0 has no duplicate-ID rule. It runs `specsync check` on a spec whose `requirements.md` repeats `REQ-auth-001`: it passes and records a clean snapshot.
  2. A build of main reports the same `6.2.0`. Its default `specsync check` prints "Skipped 1 unchanged spec(s)", then "All specs unchanged — nothing to validate", and exits 0.
  3. With `--force`, the same build prints "requirement ID `REQ-auth-001` appears 2 times" and exits 1.
- `validation_input_digest` frames `CACHE_FORMAT_VERSION`, `CARGO_PKG_VERSION` and the input files, and nothing that names the code that judged them. Every unreleased build of one version shares that identity.
- `read_cache_build_identity` is `CARGO_PKG_VERSION` plus the limits-file digest. Its doc says "a build that judges history differently never reuses another build's verdicts", which a code change at the same version breaks. The cache file sits in the Git common directory, so every worktree of the repository shares it, whatever branch it is on.
- CI uses `--strict`, which bypasses the snapshot replay, and the next version bump invalidates both caches. So the hole is local, between version bumps: exactly the multi-branch, multi-agent setup in use now.
