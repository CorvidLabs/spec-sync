---
change: reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a
artifact: docs
---

# Docs

- `CHANGELOG.md` Unreleased, Fixed: a default `specsync check` no longer replays a result cached by another build of the same version, and lifecycle reads no longer reuse another build's cached history verdicts (#878).
- Spec rows: hash_cache adds `build.rs` to `files:` and gets a Change Log row; change gets a Change Log row.
